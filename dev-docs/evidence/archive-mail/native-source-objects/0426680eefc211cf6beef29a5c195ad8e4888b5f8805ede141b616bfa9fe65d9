/* Experimental NetBSD listener-only pre-fork gate. BSD-3-Clause; see LICENSE.
 * No SSH parsing, proxy, key material, resident worker or persisted addresses.
 */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/file.h>
#include <sys/syscall.h>
#include <sys/sysctl.h>
#include <sys/time.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <limits.h>
#include <signal.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define DIRECTORY "/var/db/faction-gate"
#define STATE DIRECTORY "/admission.state"
#define LIBRARY "/usr/local/libexec/faction-ssh-gate.so"
#define SSHD "/usr/sbin/sshd"
#define MAGIC UINT64_C(0x4641435447415431)
#define SOURCES 32
struct state {
    uint64_t magic,version,refill_ms,burst,budget,last,boot_sec,boot_usec;
    uint64_t starts,passed,global_drops,source_drops,table_drops,owner,checksum;
};
_Static_assert(sizeof(struct state)==120,"fixed native registry size");
static void require(int ok) { if(!ok) _exit(1); }
static uint64_t clock_ns(void) {
    struct timespec t;require(!clock_gettime(CLOCK_MONOTONIC,&t));
    return (uint64_t)t.tv_sec*1000000000+(uint64_t)t.tv_nsec;
}
static struct timeval boot(void) {
    int mib[]={CTL_KERN,KERN_BOOTTIME};struct timeval t;size_t n=sizeof(t);
    require(!sysctl(mib,2,&t,&n,NULL,0) && n==sizeof(t));return t;
}
static uint64_t digest(const struct state *s) {
    uint64_t h=UINT64_C(14695981039346656037);const unsigned char *p=(const unsigned char *)s;
    for(size_t i=0;i<offsetof(struct state,checksum);i++) { h^=p[i];h*=UINT64_C(1099511628211); }
    return h;
}
static void parents(void) {
    const char *paths[]={"/","/var","/var/db",DIRECTORY};
    for(size_t i=0;i<sizeof(paths)/sizeof(paths[0]);i++) {
        struct stat s;require(!lstat(paths[i],&s) && S_ISDIR(s.st_mode) && s.st_uid==0 && !(s.st_mode&(i==3?077:022)));
        /* Ordinary root ancestors may be world-readable/executable. */
    }
}
static void metadata(int fd,struct stat *s) {
    require(!fstat(fd,s) && S_ISREG(s->st_mode) && s->st_uid==0 && s->st_nlink==1 && (s->st_mode&0777)==0600);
}
static void read_state(int fd,struct state *s) {
    struct stat st;metadata(fd,&st);require(st.st_size==(off_t)sizeof(*s));
    require(pread(fd,s,sizeof(*s),0)==(ssize_t)sizeof(*s));
    require(s->magic==MAGIC && s->version==1 && s->checksum==digest(s));
    require(s->refill_ms>=100 && s->refill_ms<=60000 && s->burst==8 && s->budget<=s->refill_ms*1000000*8);
}
static void persist(int fd,struct state *s,int sync) {
    s->checksum=digest(s);const unsigned char *p=(const unsigned char *)s;size_t left=sizeof(*s);off_t offset=0;
    while(left) { ssize_t n=pwrite(fd,p,left,offset);if(n<0 && errno==EINTR) continue;require(n>0);p+=n;left-=(size_t)n;offset+=n; }
    require(!ftruncate(fd,sizeof(*s)));if(sync) require(!fsync(fd));
}
#ifndef GATE_TOOL
static uint64_t refill(uint64_t budget,uint64_t last,uint64_t now,uint64_t maximum) {
    require(now>=last && budget<=maximum);uint64_t delta=now-last;
    return delta>=maximum-budget?maximum:budget+delta;
}
#endif
#ifdef GATE_TOOL
static unsigned number(const char *p) {
    char *end;errno=0;unsigned long n=strtoul(p,&end,10);
    require(*p && *p!='-' && !errno && !*end && n>=100 && n<=60000);return (unsigned)n;
}
static void stopped(void) {
    int fd=open("/var/run/faction-sshd.pid",O_RDONLY|O_CLOEXEC|O_NOFOLLOW|O_NONBLOCK);
    if(fd<0) { require(errno==ENOENT);return; }
    char b[32]={0};struct stat st;
    require(!fstat(fd,&st) && S_ISREG(st.st_mode) && st.st_uid==0 && !(st.st_mode&022) && st.st_size>0 && st.st_size<(off_t)sizeof(b));
    require(read(fd,b,(size_t)st.st_size)==st.st_size);close(fd);
    char *end;errno=0;unsigned long pid=strtoul(b,&end,10);require(!errno && pid>1 && pid<=INT_MAX && (*end==0 || (*end=='\n' && !end[1])));
    require(kill((pid_t)pid,0)<0 && errno==ESRCH);
}
int main(int argc,char **argv) {
    int inspect=0,initialize=0,reset=0,experimental=0;unsigned ms=5000;
    for(int i=1;i<argc;i++) {
        if(!strcmp(argv[i],"--inspect")) inspect=1;
        else if(!strcmp(argv[i],"--initialize")) initialize=1;
        else if(!strcmp(argv[i],"--reset")) reset=1;
        else if(!strcmp(argv[i],"--experimental-capacity")) experimental=1;
        else if(!strcmp(argv[i],"--refill-ms")) { require(++i<argc);ms=number(argv[i]); }
        else require(0);
    }
    require(getuid()==0 && geteuid()==0 && inspect+initialize+reset==1 && (experimental || ms>=5000));
    closefrom(3);umask(077);alarm(3);parents();
    int created=0,fd=open(STATE,O_RDWR|O_NOFOLLOW|O_CLOEXEC|O_NONBLOCK);
    if(fd<0 && errno==ENOENT && initialize) { stopped();fd=open(STATE,O_RDWR|O_CREAT|O_EXCL|O_NOFOLLOW|O_CLOEXEC,0600);created=fd>=0; }
    require(fd>=0);struct stat st;metadata(fd,&st);struct state s;
    if(inspect) {
        /* Diagnostic only. A live owner holds the flock. A racing short/torn
         * diagnostic read is refused by the checksum, never used as authority. */
        read_state(fd,&s);
        printf("{\"version\":1,\"state_bytes\":120,\"burst\":8,\"source_burst\":2,\"source_slots\":32,\"refill_ms\":%"PRIu64",\"source_refill_ms\":%"PRIu64",\"budget_ns\":%"PRIu64",\"last_ns\":%"PRIu64",\"starts\":%"PRIu64",\"passed\":%"PRIu64",\"global_drops\":%"PRIu64",\"source_drops\":%"PRIu64",\"table_drops\":%"PRIu64"}\n",s.refill_ms,2*s.refill_ms,s.budget,s.last,s.starts,s.passed,s.global_drops,s.source_drops,s.table_drops);return 0;
    }
    require(!flock(fd,LOCK_EX|LOCK_NB));
    if(!created) { read_state(fd,&s);if(initialize) { require(s.refill_ms==ms);return 0; } }
    stopped();memset(&s,0,sizeof(s));struct timeval t=boot();
    s.magic=MAGIC;s.version=1;s.refill_ms=ms;s.burst=8;s.budget=(uint64_t)ms*1000000*8;s.last=clock_ns();s.boot_sec=(uint64_t)t.tv_sec;s.boot_usec=(uint64_t)t.tv_usec;
    persist(fd,&s,1);return 0;
}
#else
static struct state policy;
static int authority=-1;
static dev_t device;
static ino_t inode;
static uint64_t last_flush,disk_digest;
struct source { unsigned char address[16];uint64_t budget,last;int family; };
static struct source sources[SOURCES];
static void root_listener(void) {
    require(getuid()==0 && geteuid()==0 && !strcmp(getprogname(),"sshd"));
}
static void start(void) {
    root_listener();if(authority>=0) return;
    parents();authority=open(STATE,O_RDWR|O_CLOEXEC|O_NOFOLLOW|O_NONBLOCK);require(authority>=0);
    require(!flock(authority,LOCK_EX|LOCK_NB));read_state(authority,&policy);
    struct stat st;metadata(authority,&st);device=st.st_dev;inode=st.st_ino;
    struct timeval t=boot();uint64_t now=clock_ns();
    if(policy.boot_sec!=(uint64_t)t.tv_sec || policy.boot_usec!=(uint64_t)t.tv_usec) {
        /* Reboot never manufactures a free burst. Refill conservatively from
         * an empty bucket. Normal same-boot restart retains the exact budget. */
        policy.budget=0;policy.last=now;policy.boot_sec=(uint64_t)t.tv_sec;policy.boot_usec=(uint64_t)t.tv_usec;
    }
    require(now>=policy.last);policy.owner=(uint64_t)getpid();policy.starts++;
    persist(authority,&policy,1);disk_digest=policy.checksum;last_flush=now;
}
static void finish(void) __attribute__((destructor));
static void finish(void) {
    if(authority>=0 && policy.owner==(uint64_t)getpid()) {
        struct state disk;read_state(authority,&disk);require(disk.checksum==disk_digest);
        persist(authority,&policy,1);
    }
}
int listen(int fd,int backlog) {
    start();return (int)syscall(SYS_listen,fd,backlog);
}
int accept(int fd,struct sockaddr *restrict address,socklen_t *restrict length) {
    root_listener();require(authority>=0);
    /* Keep NetBSD accept's original inherited O_NONBLOCK semantics. */
    int sock=(int)syscall(SYS_accept,fd,address,length);if(sock<0) return sock;
    struct state disk;read_state(authority,&disk);require(disk.checksum==disk_digest);
    struct stat path;
    require(!lstat(STATE,&path) && S_ISREG(path.st_mode) && path.st_dev==device && path.st_ino==inode && path.st_nlink==1 && path.st_uid==0 && (path.st_mode&0777)==0600);
    uint64_t now=clock_ns(),step=policy.refill_ms*1000000;
    policy.budget=refill(policy.budget,policy.last,now,step*8);policy.last=now;
    if(policy.budget<step) { policy.global_drops++;goto denied; }
    struct sockaddr_storage peer;socklen_t size=sizeof(peer);require(!getpeername(sock,(struct sockaddr *)&peer,&size));
    unsigned char key[16]={0};int family=peer.ss_family;
    if(family==AF_INET) memcpy(key,&((struct sockaddr_in *)&peer)->sin_addr,4);
    else if(family==AF_INET6) {
        struct in6_addr *v=&((struct sockaddr_in6 *)&peer)->sin6_addr;
        if(IN6_IS_ADDR_V4MAPPED(v)) { family=AF_INET;memcpy(key,&v->s6_addr[12],4); }
        else memcpy(key,v,16);
    } else { close(sock);errno=EAFNOSUPPORT;return -1; }
    unsigned match=SOURCES,empty=SOURCES;uint64_t source_step=step*2;
    for(unsigned i=0;i<SOURCES;i++) {
        struct source *s=&sources[i];
        if(s->family) { s->budget=refill(s->budget,s->last,now,source_step*2);s->last=now; }
        if(s->family==family && !memcmp(s->address,key,16)) match=i;
        if(!s->family || s->budget==source_step*2) empty=i;
    }
    if(match==SOURCES) {
        if(empty==SOURCES) { policy.table_drops++;goto denied; }
        match=empty;sources[match]=(struct source){.budget=source_step*2,.last=now,.family=family};memcpy(sources[match].address,key,16);
    }
    if(sources[match].budget<source_step) { policy.source_drops++;goto denied; }
    sources[match].budget-=source_step;policy.budget-=step;policy.passed++;
    /* The global rate budget is durable before OpenSSH can fork. */
    persist(authority,&policy,1);disk_digest=policy.checksum;last_flush=now;return sock;
denied:
    /* At most one unsynchronized diagnostic update/second; no per-drop fsync.
     * A crash may lose diagnostic drops, never create extra global tokens. */
    if(now-last_flush>=1000000000) { persist(authority,&policy,0);disk_digest=policy.checksum;last_flush=now; }
    close(sock);errno=EAGAIN;return -1;
}
extern char **environ;
int execv(const char *path,char *const argv[]) {
    root_listener();
    /* HUP reexec must fail closed, never restart without the gate. The profile
     * deliberately defines HUP as listener stop; explicit rc restart is used. */
    if(!strcmp(path,SSHD)) { finish();_exit(0); }
    require(!strcmp(path,"/usr/libexec/sshd-session"));
    require(!unsetenv("LD_PRELOAD") && !unsetenv("LD_BIND_NOW"));
    return (int)syscall(SYS_execve,path,argv,environ);
}
#endif
