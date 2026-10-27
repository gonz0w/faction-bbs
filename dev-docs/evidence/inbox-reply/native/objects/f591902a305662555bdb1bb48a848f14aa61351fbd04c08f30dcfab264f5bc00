/* Experimental NetBSD SSH admission. BSD-3-Clause; see LICENSE.
 * No SSH parsing, proxy, address history or dynamic connection allocation.
 * Each fixed slot owns the original sshd -i PID until waitpid reaps it.
 */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/file.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <limits.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define MAX_CONNECTIONS 250
#define CONFIG "/etc/ssh/faction/admission_sshd_config"
#define PIDFILE "/var/run/faction-sshd.pid"
#define STATUS "/var/run/faction-admission.status"
static volatile sig_atomic_t stopping;
static pid_t children[MAX_CONNECTIONS];
static unsigned active, peak, capacity=8, burst=8, refill_ms=5000;
static uint64_t launches, full, rate, failed;
static int statusfd=-1;
static void die(const char *why) { perror(why); exit(1); }
static void check(int yes,const char *why) { if(!yes) { errno=EINVAL;die(why); } }
static void stop_signal(int signo) { (void)signo;stopping=1; }
static void child_signal(int signo) { (void)signo; }
static uint64_t now_ns(void) {
    struct timespec t;if(clock_gettime(CLOCK_MONOTONIC,&t)) die("monotonic clock");
    return (uint64_t)t.tv_sec*1000000000+(uint64_t)t.tv_nsec;
}
static unsigned number(const char *s,unsigned max) {
    char *end;errno=0;unsigned long n=strtoul(s,&end,10);
    check(*s && *s!='-' && !errno && !*end && n && n<=max,"numeric option");
    return (unsigned)n;
}
/* All parents must be actual root-owned directories, never writable by peers. */
static void parents(const char *path) {
    char copy[PATH_MAX];check(strlen(path)<sizeof(copy),"path length");strcpy(copy,path);
    for(char *p=copy+1;*p;p++) if(*p=='/') {
        *p=0;struct stat s;
        check(!lstat(copy,&s) && S_ISDIR(s.st_mode) && s.st_uid==0 && !(s.st_mode&022),"protected parent");
        *p='/';
    }
}
static int private_file(const char *path,int create) {
    parents(path);int fd=open(path,O_RDWR|O_CLOEXEC|O_NOFOLLOW|(create?O_CREAT:0),0600);
    if(fd<0) die("private file");
    struct stat s;
    check(!fstat(fd,&s) && S_ISREG(s.st_mode) && s.st_uid==0 && s.st_nlink==1 && (s.st_mode&0777)==0600,"private file metadata");
    return fd;
}
static void put(int fd,const char *s,size_t n) {
    check(lseek(fd,0,SEEK_SET)==0,"file rewind");
    while(n) { ssize_t k=write(fd,s,n);if(k<0 && errno==EINTR) continue;if(k<=0) die("file write");s+=k;n-=(size_t)k; }
    off_t end=lseek(fd,0,SEEK_CUR);check(end>=0 && !ftruncate(fd,end),"file truncate");
}
static void statistics(void) {
    char b[512];int n=snprintf(b,sizeof(b),"{\"capacity\":%u,\"burst\":%u,\"refill_ms\":%u,\"active\":%u,\"peak\":%u,\"launches\":%"PRIu64",\"full\":%"PRIu64",\"rate\":%"PRIu64",\"fork_failures\":%"PRIu64"}\n",capacity,burst,refill_ms,active,peak,launches,full,rate,failed);
    check(n>0 && (size_t)n<sizeof(b),"status length");put(statusfd,b,(size_t)n);
}
static void reap(void) {
    for(;;) {
        pid_t p=waitpid(-1,NULL,WNOHANG);
        if(p<0 && errno==EINTR) continue;
        if(p<=0) { check(p==0 || errno==ECHILD,"waitpid");break; }
        unsigned i;for(i=0;i<capacity;i++) if(children[i]==p) { children[i]=0;active--;break; }
        check(i<capacity,"untracked child");
    }
}
static void signal_children(int signo) {
    for(unsigned i=0;i<capacity;i++) if(children[i]) {
        /* Before setsid the group may not exist yet; signal the PID as well. */
        (void)kill(-children[i],signo);(void)kill(children[i],signo);
    }
}
static void serve(int fd) {
    struct sigaction sa;memset(&sa,0,sizeof(sa));sa.sa_handler=SIG_DFL;sigemptyset(&sa.sa_mask);
    if(sigaction(SIGTERM,&sa,NULL) || sigaction(SIGINT,&sa,NULL) || sigaction(SIGCHLD,&sa,NULL) || sigaction(SIGHUP,&sa,NULL)) _exit(126);
    if(setsid()<0) _exit(126);
    int f=fcntl(fd,F_GETFL);if(f<0 || fcntl(fd,F_SETFL,f&~O_NONBLOCK)<0) _exit(126);
    if(dup2(fd,0)<0 || dup2(fd,1)<0 || dup2(fd,2)<0) _exit(126);
    closefrom(3);
    char *argv[]={"/usr/sbin/sshd","-i","-f",CONFIG,NULL};
    char *env[]={"PATH=/usr/sbin:/sbin:/usr/bin:/bin","LANG=C",NULL};
    execve(argv[0],argv,env);_exit(126);
}
int main(int argc,char **argv) {
    const char *address="127.0.0.1";unsigned port=2222;int experimental=0,foreground=0;
    for(int i=1;i<argc;i++) {
        if(!strcmp(argv[i],"--experimental-capacity")) { experimental=1;continue; }
        if(!strcmp(argv[i],"--foreground")) { foreground=1;continue; }
        check(i+1<argc,"option value");const char *key=argv[i++],*value=argv[i];
        if(!strcmp(key,"--listen")) address=value;
        else if(!strcmp(key,"--port")) port=number(value,65535);
        else if(!strcmp(key,"--capacity")) capacity=number(value,MAX_CONNECTIONS);
        else if(!strcmp(key,"--burst")) burst=number(value,8);
        else if(!strcmp(key,"--refill-ms")) refill_ms=number(value,60000);
        else check(0,"unknown option");
    }
    check(getuid()==0 && geteuid()==0,"root required");
    check(refill_ms>=100 && (experimental || (capacity<=8 && refill_ms>=5000)),"experimental admission requires explicit flag");
    umask(077);closefrom(3);
    struct rlimit core={0,0};check(!setrlimit(RLIMIT_CORE,&core),"core limit");
    int config=private_file(CONFIG,0);close(config);
    int pidfd=private_file(PIDFILE,1);check(!flock(pidfd,LOCK_EX|LOCK_NB),"admission lock");
    char old[32];ssize_t n=read(pidfd,old,sizeof(old)-1);check(n>=0 && n<(ssize_t)sizeof(old)-1,"pid record length");old[n]=0;
    /* Nonzero records are dirty even if that PID is dead. Never reset after a
     * crash until an operator proves all old SSH descendants have gone. */
    check(n==0 || !strcmp(old,"0\n"),"unclean or competing SSH pid record");
    struct sockaddr_in sa;memset(&sa,0,sizeof(sa));sa.sin_len=sizeof(sa);sa.sin_family=AF_INET;sa.sin_port=htons((uint16_t)port);
    check(inet_pton(AF_INET,address,&sa.sin_addr)==1,"numeric IPv4 listener");
    int listener=socket(AF_INET,SOCK_STREAM|SOCK_CLOEXEC|SOCK_NONBLOCK,0);if(listener<0) die("listener");
    int yes=1;check(!setsockopt(listener,SOL_SOCKET,SO_REUSEADDR,&yes,sizeof(yes)),"reuse address");
    check(!bind(listener,(struct sockaddr *)&sa,sizeof(sa)) && !listen(listener,8),"bind/listen");
    statusfd=private_file(STATUS,1);
    if(!foreground && daemon(0,0)) die("daemon");
    char pid[32];n=snprintf(pid,sizeof(pid),"%ld\n",(long)getpid());check(n>0 && (size_t)n<sizeof(pid),"pid length");put(pidfd,pid,(size_t)n);
    struct sigaction action;memset(&action,0,sizeof(action));sigemptyset(&action.sa_mask);action.sa_handler=stop_signal;
    check(!sigaction(SIGTERM,&action,NULL) && !sigaction(SIGINT,&action,NULL) && !sigaction(SIGHUP,&action,NULL),"stop handlers");
    action.sa_handler=child_signal;check(!sigaction(SIGCHLD,&action,NULL),"child handler");
    uint64_t step=(uint64_t)refill_ms*1000000,maximum=step*burst,budget=maximum,last=now_ns(),reported=last;
    statistics();
    while(!stopping) {
        reap();uint64_t now=now_ns(),delta=now-last;last=now;
        budget=delta>=maximum-budget?maximum:budget+delta;
        if(now-reported>=1000000000) { statistics();reported=now; }
        struct pollfd pollfd={listener,POLLIN,0};int ready=poll(&pollfd,1,100);
        if(ready<0 && errno==EINTR) continue;
        check(ready>=0 && !(pollfd.revents&(POLLERR|POLLHUP|POLLNVAL)),"listener poll");
        if(!ready) continue;
        /* A flood cannot indefinitely defer reaping, stop signals or refill. */
        for(unsigned batch=0;batch<128 && !stopping;batch++) {
            int fd=accept(listener,NULL,NULL);
            if(fd<0 && (errno==EAGAIN || errno==EWOULDBLOCK || errno==EINTR || errno==ECONNABORTED)) break;
            if(fd<0) die("accept");
            if(active>=capacity) { full++;close(fd);continue; }
            if(budget<step) { rate++;close(fd);continue; }
            unsigned slot=0;while(slot<capacity && children[slot]) slot++;
            check(slot<capacity,"connection table");
            pid_t child=fork();
            if(child==0) serve(fd);
            close(fd);
            if(child<0) { failed++;continue; }
            children[slot]=child;active++;if(active>peak) peak=active;
            launches++;budget-=step;
        }
    }
    close(listener);signal_children(SIGTERM);uint64_t end=now_ns()+5000000000;
    while(active && now_ns()<end) { reap();(void)poll(NULL,0,20); }
    if(active) signal_children(SIGKILL);
    end=now_ns()+2000000000;
    while(active && now_ns()<end) { reap();(void)poll(NULL,0,20); }
    statistics();check(!active,"SSH children did not reap");
    put(pidfd,"0\n",2);close(statusfd);close(pidfd);return 0;
}
