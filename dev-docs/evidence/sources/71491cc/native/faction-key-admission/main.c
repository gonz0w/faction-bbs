/* Experimental base-OpenSSH AuthorizedKeysCommand. BSD-3-Clause; see LICENSE.
 * Fixed process-birth registry; no key/SSH parser, network I/O or resident daemon.
 */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/sysctl.h>
#include <sys/proc.h>
#include <sys/file.h>
#include <sys/resource.h>
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pwd.h>
#include <signal.h>
#include <time.h>
#include <unistd.h>

#define CAP 250
#define MAGIC UINT64_C(0x464143544b455931)
#define DIRECTORY "/var/run/faction-auth"
#define STATE DIRECTORY "/admission.state"
#define KEYS "/etc/ssh/faction/authorized_keys"
struct identity { uint64_t pid,sec,usec; };
struct state {
    uint64_t magic,version,capacity,refill_ms,burst,budget,last;
    uint64_t launches,reuses,full,rate,peak;
    struct identity slots[CAP];
    uint64_t checksum;
};
_Static_assert(sizeof(struct state)==offsetof(struct state,checksum)+8,"no trailing padding");
static void require(int ok) { if(!ok) _exit(1); }
static uint64_t clock_ns(void) {
    struct timespec t;require(!clock_gettime(CLOCK_MONOTONIC,&t));
    return (uint64_t)t.tv_sec*1000000000+(uint64_t)t.tv_nsec;
}
static uint64_t digest(const struct state *s) {
    const unsigned char *p=(const unsigned char *)s;uint64_t h=UINT64_C(14695981039346656037);
    for(size_t i=0;i<offsetof(struct state,checksum);i++) { h^=p[i];h*=UINT64_C(1099511628211); }
    return h;
}
static unsigned number(const char *p,unsigned maximum) {
    char *end;errno=0;unsigned long n=strtoul(p,&end,10);
    require(*p && *p!='-' && !errno && !*end && n && n<=maximum);return (unsigned)n;
}
/* No hidden-process policy: absence must mean death, not filtered visibility. */
static void visibility(void) {
    int curtain=-1;size_t n=sizeof(curtain);
    require(!sysctlbyname("security.curtain",&curtain,&n,NULL,0) && n==sizeof(curtain) && curtain==0);
}
static int process(uint64_t pid,struct kinfo_proc2 *p) {
    require(pid>1 && pid<=INT_MAX);
    int mib[]={CTL_KERN,KERN_PROC2,KERN_PROC_PID,(int)pid,sizeof(*p),1};size_t n=sizeof(*p);
    if(sysctl(mib,6,p,&n,NULL,0)) { require(errno==ESRCH);return 0; }
    if(n==0) return 0;
    require(n==sizeof(*p) && p->p_pid==(pid_t)pid);return p->p_stat!=SZOMB && p->p_stat!=SDEAD;
}
/* OpenSSH temporarily uses the helper effective UID while collecting output.
 * Its immutable real-root identity and birth time must survive that transition. */
static int live(struct identity id) {
    if(!id.pid) { require(!id.sec && !id.usec);return 0; }
    struct kinfo_proc2 p;
    return process(id.pid,&p) && p.p_ruid==0 && !strcmp(p.p_comm,"sshd-session") &&
        (uint64_t)p.p_ustart_sec==id.sec && (uint64_t)p.p_ustart_usec==id.usec;
}
static unsigned prune(struct state *s) {
    unsigned n=0;
    for(unsigned i=0;i<CAP;i++) {
        if(live(s->slots[i])) n++;
        else memset(&s->slots[i],0,sizeof(s->slots[i]));
    }
    return n;
}
static void parents(void) {
    const char *root[]={"/","/var","/var/run","/etc","/etc/ssh","/etc/ssh/faction"};
    for(size_t i=0;i<sizeof(root)/sizeof(root[0]);i++) {
        struct stat s;require(!lstat(root[i],&s) && S_ISDIR(s.st_mode) && s.st_uid==0 && !(s.st_mode&022));
    }
}
static void persist(int fd,struct state *s) {
    s->checksum=digest(s);const unsigned char *p=(const unsigned char *)s;size_t left=sizeof(*s);off_t offset=0;
    while(left) {
        ssize_t n=pwrite(fd,p,left,offset);if(n<0 && errno==EINTR) continue;
        require(n>0);p+=n;left-=(size_t)n;offset+=n;
    }
    require(!ftruncate(fd,sizeof(*s)) && !fsync(fd));
}
static void fresh(struct state *s,unsigned capacity,unsigned refill) {
    memset(s,0,sizeof(*s));s->magic=MAGIC;s->version=1;s->capacity=capacity;s->refill_ms=refill;s->burst=8;
    s->budget=(uint64_t)refill*1000000*8;s->last=clock_ns();
}
int main(int argc,char **argv) {
    unsigned capacity=8,refill=5000,target=0;int experimental=0,inspect=0,reset=0;
    for(int i=1;i<argc;i++) {
        if(!strcmp(argv[i],"--experimental-capacity")) { experimental=1;continue; }
        if(!strcmp(argv[i],"--inspect")) { inspect=1;continue; }
        if(!strcmp(argv[i],"--reset")) { reset=1;continue; }
        require(i+1<argc);const char *key=argv[i++],*value=argv[i];
        if(!strcmp(key,"--capacity")) capacity=number(value,CAP);
        else if(!strcmp(key,"--refill-ms")) refill=number(value,60000);
        else if(!strcmp(key,"--target-uid")) target=number(value,INT_MAX);
        else require(0);
    }
    require(refill>=100 && (experimental || (capacity<=8 && refill>=5000)) && !(inspect && reset));
    closefrom(3);umask(077);alarm(3);
    struct rlimit zero={0,0};require(!setrlimit(RLIMIT_CORE,&zero) && !setrlimit(RLIMIT_MEMLOCK,&zero));
    struct passwd *pw=getpwnam("factionauth");require(pw && pw->pw_uid>0);uid_t helper=pw->pw_uid;
    pw=getpwnam("factionterm");require(pw && pw->pw_uid>0 && pw->pw_uid!=helper);uid_t terminal=pw->pw_uid;
    require((inspect || reset)?(getuid()==0 && geteuid()==0):(getuid()==helper && geteuid()==helper && target==terminal));
    visibility();parents();
    struct stat dir;require(!lstat(DIRECTORY,&dir) && S_ISDIR(dir.st_mode) && dir.st_uid==helper && (dir.st_mode&0777)==0700);
    struct identity caller={0,0,0};
    if(!inspect && !reset) {
        struct kinfo_proc2 parent;pid_t pid=getppid();require(process((uint64_t)pid,&parent));
        require(parent.p_ruid==0 && !strcmp(parent.p_comm,"sshd-session"));
        caller=(struct identity){(uint64_t)pid,(uint64_t)parent.p_ustart_sec,(uint64_t)parent.p_ustart_usec};
    }
    int created=0,fd=open(STATE,O_RDWR|O_CLOEXEC|O_NOFOLLOW|O_NONBLOCK);
    if(fd<0 && errno==ENOENT && reset) {
        fd=open(STATE,O_RDWR|O_CREAT|O_EXCL|O_CLOEXEC|O_NOFOLLOW,0600);
        if(fd>=0) { created=1;if(geteuid()==0) require(!fchown(fd,helper,(gid_t)-1)); }
        else if(errno==EEXIST) fd=open(STATE,O_RDWR|O_CLOEXEC|O_NOFOLLOW|O_NONBLOCK);
    }
    require(fd>=0);struct stat st;require(!fstat(fd,&st) && S_ISREG(st.st_mode) && st.st_uid==helper && st.st_nlink==1 && (st.st_mode&0777)==0600);
    require(!flock(fd,LOCK_EX));
    struct state s;memset(&s,0,sizeof(s));
    if(created) fresh(&s,capacity,refill);
    else {
        require(!fstat(fd,&st) && st.st_size==(off_t)sizeof(s));
        require(pread(fd,&s,sizeof(s),0)==(ssize_t)sizeof(s));
        require(s.magic==MAGIC && s.version==1 && s.checksum==digest(&s));
        require(s.capacity && s.capacity<=CAP && s.refill_ms>=100 && s.refill_ms<=60000 && s.burst==8);
        require(s.budget<=s.refill_ms*1000000*8);
    }
    unsigned active=prune(&s);
    if(inspect) {
        printf("{\"capacity\":%"PRIu64",\"burst\":%"PRIu64",\"refill_ms\":%"PRIu64",\"active\":%u,\"peak\":%"PRIu64",\"launches\":%"PRIu64",\"reuses\":%"PRIu64",\"full\":%"PRIu64",\"rate\":%"PRIu64"}\n",s.capacity,s.burst,s.refill_ms,active,s.peak,s.launches,s.reuses,s.full,s.rate);
        return 0;
    }
    if(reset) { require(!active);fresh(&s,capacity,refill);persist(fd,&s);return 0; }
    if(s.capacity!=capacity || s.refill_ms!=refill) { require(!active);fresh(&s,capacity,refill); }
    uint64_t now=clock_ns(),maximum=s.refill_ms*1000000*8,step=s.refill_ms*1000000;
    /* A reboot may preserve this inode while resetting monotonic time. Never
     * reset a budget if any recorded SSH monitor is still alive. */
    if(now<s.last) { require(!active);s.last=now;s.budget=maximum; }
    uint64_t delta=now-s.last;s.last=now;
    s.budget=delta>=maximum-s.budget?maximum:s.budget+delta;
    unsigned slot=CAP;int repeated=0;
    for(unsigned i=0;i<CAP;i++) if(s.slots[i].pid==caller.pid && s.slots[i].sec==caller.sec && s.slots[i].usec==caller.usec) { repeated=1;break; }
    if(repeated) s.reuses++;
    else {
        if(active>=capacity) { s.full++;persist(fd,&s);return 1; }
        if(s.budget<step) { s.rate++;persist(fd,&s);return 1; }
        for(unsigned i=0;i<CAP;i++) if(!s.slots[i].pid) { slot=i;break; }
        require(slot<CAP);s.slots[slot]=caller;active++;s.launches++;s.budget-=step;
        if(active>s.peak) s.peak=active;
    }
    /* Read bounded, root-managed key lines unchanged. OpenSSH alone parses and
     * validates the offered key/signature and the restrict,pty options. */
    int keys=open(KEYS,O_RDONLY|O_CLOEXEC|O_NOFOLLOW|O_NONBLOCK);require(keys>=0);
    require(!fstat(keys,&st) && S_ISREG(st.st_mode) && st.st_uid==0 && st.st_nlink==1 && !(st.st_mode&022) && st.st_size>0 && st.st_size<=65536);
    unsigned char data[65536];size_t length=0;
    while(length<(size_t)st.st_size) {
        ssize_t n=read(keys,data+length,(size_t)st.st_size-length);if(n<0 && errno==EINTR) continue;
        require(n>0);length+=(size_t)n;
    }
    close(keys);persist(fd,&s);require(!flock(fd,LOCK_UN));close(fd);
    size_t offset=0;while(offset<length) {
        ssize_t n=write(STDOUT_FILENO,data+offset,length-offset);if(n<0 && errno==EINTR) continue;
        require(n>0);offset+=(size_t)n;
    }
    return 0;
}
