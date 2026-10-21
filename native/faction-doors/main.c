/* BSD-3-Clause. Optional separate-UID Lua evaluator; no BBS state/TTY access. */
#define _NETBSD_SOURCE 1
#define _DEFAULT_SOURCE 1
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/resource.h>
#include <sys/file.h>
#include <sys/wait.h>
#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <sys/stat.h>

#define FRAME 4096
#define SOURCE 16384
static volatile sig_atomic_t stopping;
static void stop(int n) { (void)n; stopping=1; }
static int64_t milliseconds(void) {
    struct timespec t; if(clock_gettime(CLOCK_MONOTONIC,&t))_exit(1);
    return (int64_t)t.tv_sec*1000+t.tv_nsec/1000000;
}
static int printable(const char *s,size_t n) {
    for(size_t i=0;i<n;i++)if((unsigned char)s[i]<32 || (unsigned char)s[i]>126 || s[i]=='|')return 0;
    return 1;
}
static int slug(const char *s) {
    size_t n=strlen(s);if(!n || n>32)return 0;
    for(size_t i=0;i<n;i++)if(!((s[i]>='a' && s[i]<='z') || (s[i]>='0' && s[i]<='9') || s[i]=='-'))return 0;
    return 1;
}
static int ceiling(int which,rlim_t value) {
    struct rlimit r={value,value};return setrlimit(which,&r);
}
typedef struct {
    size_t live;unsigned ticks,lines,options;int title,prompt;char output[FRAME],state[1025];size_t length;
} Budget;
static void *allocate(void *ud,void *ptr,size_t old,size_t size) {
    Budget *b=ud;if(!ptr)old=0;
    if(!size){free(ptr);b->live-=old;return NULL;}
    if(old>b->live || size>1048576-(b->live-old))return NULL;
    void *next=realloc(ptr,size);if(next){b->live=b->live-old+size;}return next;
}
static Budget *budget(lua_State *L) { void *ud;lua_getallocf(L,&ud);return ud; }
static void tick(lua_State *L,lua_Debug *ar) { (void)ar;if(++budget(L)->ticks>=100)luaL_error(L,"instruction limit"); }
static const char *field(lua_State *L,int index,size_t maximum) {
    size_t n;const char *s=luaL_checklstring(L,index,&n);
    if(n>maximum || !printable(s,n))luaL_error(L,"invalid field");
    return s;
}
static void append(lua_State *L,const char *text) {
    Budget *b=budget(L);size_t n=strlen(text);
    if(b->length+n>=FRAME-1100)luaL_error(L,"screen limit");
    memcpy(b->output+b->length,text,n+1);b->length+=n;
}
static int title(lua_State *L) {
    if(budget(L)->title++)return luaL_error(L,"duplicate title");
    char line[80];snprintf(line,sizeof line,"TITLE|%s\n",field(L,1,64));append(L,line);return 0;
}
static int text(lua_State *L) {
    if(++budget(L)->lines>16)return luaL_error(L,"line limit");
    char line[140];snprintf(line,sizeof line,"TEXT|%s\n",field(L,1,120));append(L,line);return 0;
}
static int prompt(lua_State *L) {
    if(budget(L)->prompt++)return luaL_error(L,"duplicate prompt");
    char line[140];snprintf(line,sizeof line,"PROMPT|%s\n",field(L,1,120));append(L,line);return 0;
}
static int option(lua_State *L) {
    if(++budget(L)->options>8)return luaL_error(L,"option limit");
    const char *key=field(L,1,1),*label=field(L,2,48),*target=field(L,3,64);
    if(strlen(key)!=1 || key[0]<'A' || key[0]>'Z')return luaL_error(L,"invalid hotkey");
    char line[130];snprintf(line,sizeof line,"OPTION|%s|%s|%s\n",key,label,target);append(L,line);return 0;
}
static int state(lua_State *L) { snprintf(budget(L)->state,sizeof budget(L)->state,"%s",field(L,1,1024));return 0; }
static int random_number(lua_State *L) {
    lua_Integer maximum=luaL_checkinteger(L,1);if(maximum<1 || maximum>1000000)return luaL_error(L,"random range");
    lua_pushinteger(L,1+arc4random_uniform((uint32_t)maximum));return 1;
}
static int number(lua_State *L) { int ok;lua_Number n=lua_tonumberx(L,1,&ok);if(ok)lua_pushnumber(L,n);else lua_pushnil(L);return 1; }
static void evaluate(const char *source,size_t length,const char *previous,const char *input,int output) {
    signal(SIGTERM,SIG_DFL);signal(SIGHUP,SIG_DFL);signal(SIGINT,SIG_DFL);
    if(ceiling(RLIMIT_CORE,0) || ceiling(RLIMIT_MEMLOCK,0) || ceiling(RLIMIT_NPROC,0) ||
       ceiling(RLIMIT_NOFILE,8) || ceiling(RLIMIT_CPU,1) ||
       ceiling(RLIMIT_DATA,16*1024*1024) || ceiling(RLIMIT_STACK,8*1024*1024))_exit(1);
#ifdef __NetBSD__
    if(ceiling(RLIMIT_NTHR,0))_exit(1);
#endif
    Budget b={0};memcpy(b.output,"FDS1\n",6);b.length=5;
#if LUA_VERSION_NUM >= 505
    lua_State *L=lua_newstate(allocate,&b,arc4random());
#else
    lua_State *L=lua_newstate(allocate,&b);
#endif
    if(!L)_exit(1);
    /* No base/package/os/io/debug/coroutine libraries or loaders. String
       operations are bounded by the allocator and the parent's wall deadline. */
    luaL_requiref(L,"string",luaopen_string,1);lua_pop(L,1);
    lua_pushcfunction(L,number);lua_setglobal(L,"tonumber");
    lua_pushstring(L,previous);lua_setglobal(L,"state");
    lua_pushstring(L,input);lua_setglobal(L,"input");
    lua_newtable(L);
    const luaL_Reg api[]={{"title",title},{"text",text},{"prompt",prompt},{"option",option},
        {"state",state},{"random",random_number},{NULL,NULL}};
    luaL_setfuncs(L,api,0);lua_setglobal(L,"bbs");lua_sethook(L,tick,LUA_MASKCOUNT,1000);
    int status=luaL_loadbufferx(L,source,length,"door","t");if(!status)status=lua_pcall(L,0,0,0);
    if(!status && b.title==1 && (b.prompt || b.options)) {
        int n=snprintf(b.output+b.length,sizeof b.output-b.length,"STATE|%s\nEND\n",b.state);
        if(n>0 && (size_t)n<sizeof b.output-b.length) {
            b.length+=(size_t)n;size_t sent=0;
            while(sent<b.length){ssize_t count=write(output,b.output+sent,b.length-sent);if(count<=0)break;sent+=(size_t)count;}
            status=sent==b.length?0:1;
        } else status=1;
    } else status=1;
    lua_close(L);_exit(status?1:0);
}
static int peer(int fd,uid_t owner) {
#ifdef __linux__
    struct ucred cred;socklen_t length=sizeof cred;
    return getsockopt(fd,SOL_SOCKET,SO_PEERCRED,&cred,&length)==0 && length==sizeof cred && cred.uid==owner;
#else
    uid_t uid;gid_t gid;return getpeereid(fd,&uid,&gid)==0 && uid==owner;
#endif
}
static int read_request(int fd,char *buffer) {
    size_t n=0;int64_t end=milliseconds()+500;
    while(n<FRAME-1 && milliseconds()<end && !stopping) {
        int wait=(int)(end-milliseconds());if(wait<=0)return -1;
        struct pollfd p={fd,POLLIN,0};int ready=poll(&p,1,wait);
        if(ready<=0)return -1;
        ssize_t count=read(fd,buffer+n,FRAME-1-n);
        if(count<0)return -1;
        if(!count){buffer[n]=0;return (int)n;}
        if(memchr(buffer+n,0,(size_t)count))return -1;
        n+=(size_t)count;
    }
    return -1;
}
static void serve(int fd,int scripts) {
    char request[FRAME],source[SOURCE+1];int count=read_request(fd,request);
    if(count<=0 || request[count-1]!='\n')return;
    request[count-1]=0;
    char *first=strchr(request,'|');if(!first)return;*first++=0;
    char *second=strchr(first,'|');if(!second)return;*second++=0;
    if(!slug(request) || strlen(first)>1024 || strlen(second)>128 ||
       !printable(first,strlen(first)) || !printable(second,strlen(second)))return;
    char filename[40];snprintf(filename,sizeof filename,"%s.lua",request);
    int script=openat(scripts,filename,O_RDONLY|O_NOFOLLOW|O_NONBLOCK|O_CLOEXEC);if(script<0)return;
    struct stat st;if(fstat(script,&st) || !S_ISREG(st.st_mode) || st.st_nlink!=1 || st.st_uid!=0 ||
        (st.st_mode&022) || st.st_size<1 || st.st_size>SOURCE){close(script);return;}
    ssize_t length=read(script,source,sizeof source);close(script);
    if(length!=st.st_size)return;
    int pipes[2];if(pipe(pipes))return;
    pid_t child=fork();
    if(child==0){close(pipes[0]);close(fd);close(scripts);
        /* Only a write-only reply pipe survives beyond stdio. The inherited
           listener and pidfile are closed before entering Lua. */
        int out=dup2(pipes[1],3);if(out<0)_exit(1);
        int null=open("/dev/null",O_RDWR);if(null<0)_exit(1);
        for(int i=0;i<3;i++)if(dup2(null,i)<0)_exit(1);
#ifdef __APPLE__
        for(int i=4;i<getdtablesize();i++)close(i);
#else
        closefrom(4);
#endif
        evaluate(source,(size_t)length,first,second,out);
    }
    close(pipes[1]);if(child<0){close(pipes[0]);return;}
    char response[FRAME];size_t used=0;int ended=0;int64_t deadline=milliseconds()+250;
    while(milliseconds()<deadline && !stopping) {
        struct pollfd p[2]={{pipes[0],POLLIN|POLLHUP,0},{fd,POLLHUP,0}};
        int wait=(int)(deadline-milliseconds());if(wait<=0)break;
        int n=poll(p,2,wait);
        if(n<0){if(errno==EINTR)continue;break;}
        if(p[1].revents&(POLLHUP|POLLERR))break;
        if(p[0].revents&(POLLIN|POLLHUP)) {
            ssize_t got=read(pipes[0],response+used,sizeof response-used);
            if(got==0){ended=1;break;}
            if(got<0)break;
            used+=(size_t)got;if(used==sizeof response)break;
        }
    }
    close(pipes[0]);int status=0;
    if(!ended || stopping)kill(child,SIGKILL);
    while(waitpid(child,&status,0)<0){if(errno!=EINTR)return;}
    if(ended && WIFEXITED(status) && WEXITSTATUS(status)==0 && used) {
        /* A small response, and no unbounded blocking behind a member/owner. */
        struct pollfd p={fd,POLLOUT,0};if(poll(&p,1,100)>0)(void)send(fd,response,used,MSG_DONTWAIT);
    }
}
int main(int argc,char **argv) {
    if((argc!=7 && argc!=8) || strcmp(argv[1],"--socket") || strcmp(argv[3],"--owner-uid") ||
       strcmp(argv[5],"--scripts") || (argc==8 && strcmp(argv[7],"--daemon")) ||
       getuid()==0 || getuid()!=geteuid() || getgid()!=getegid())return 2;
    char *tail;unsigned long owner=strtoul(argv[4],&tail,10);if(!owner || *tail || owner>UINT32_MAX)return 2;
    if(strlen(argv[2])>=sizeof(((struct sockaddr_un *)0)->sun_path))return 2;
    if(argc==8 && daemon(0,1))return 2;
    signal(SIGPIPE,SIG_IGN);signal(SIGTERM,stop);signal(SIGINT,stop);signal(SIGHUP,stop);
    if(ceiling(RLIMIT_CORE,0) || ceiling(RLIMIT_MEMLOCK,0))return 2;
    int scripts=open(argv[6],O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);struct stat st;
    if(scripts<0 || fstat(scripts,&st) || st.st_uid!=0 || (st.st_mode&022))return 2;
    char pidpath[sizeof(((struct sockaddr_un *)0)->sun_path)+8];snprintf(pidpath,sizeof pidpath,"%s.pid",argv[2]);
    int pidfd=open(pidpath,O_RDWR|O_CREAT|O_NOFOLLOW|O_CLOEXEC,0600);
    if(pidfd<0 || fstat(pidfd,&st) || !S_ISREG(st.st_mode) || st.st_uid!=getuid() || st.st_nlink!=1 ||
       (st.st_mode&077) || flock(pidfd,LOCK_EX|LOCK_NB))return 2;
    if(lstat(argv[2],&st)==0){if(!S_ISSOCK(st.st_mode) || st.st_uid!=getuid())return 2;unlink(argv[2]);}
    else if(errno!=ENOENT)return 2;
    int listener=socket(AF_UNIX,SOCK_STREAM,0);if(listener<0)return 2;
    struct sockaddr_un address={0};address.sun_family=AF_UNIX;snprintf(address.sun_path,sizeof address.sun_path,"%s",argv[2]);
    if(bind(listener,(struct sockaddr *)&address,sizeof address) || chmod(argv[2],0660) || listen(listener,8))return 2;
    if(ftruncate(pidfd,0) || dprintf(pidfd,"%ld\n",(long)getpid())<0 || fsync(pidfd))return 2;
    while(!stopping) {
        struct pollfd p={listener,POLLIN,0};if(poll(&p,1,100)<=0)continue;
        int fd=accept(listener,NULL,NULL);if(fd<0)continue;
        if(peer(fd,(uid_t)owner))serve(fd,scripts);
        close(fd);
    }
    unlink(argv[2]);int result=ftruncate(pidfd,0);close(pidfd);close(listener);close(scripts);return result==0?0:1;
}
