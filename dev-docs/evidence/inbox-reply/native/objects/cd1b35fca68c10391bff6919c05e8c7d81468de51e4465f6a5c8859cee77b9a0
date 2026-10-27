/* BSD-3-Clause. Opt-in NetBSD pre-main isolation boundary. See plan 0021.
 * Trusted root policy and ELF loader/libc precede this initializer. Application
 * parsing runs only after chroot, full ID drop and descriptor/env sanitization.
 */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/param.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <sys/resource.h>
#include <sys/sysctl.h>
#include <errno.h>
#include <fcntl.h>
#include <grp.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

extern char **environ;
#ifdef FACTION_TERMINAL
static char term_env[69];
static char *terminal_env[]={term_env,"SSH_CONNECTION=isolated terminal",NULL};
#else
static char *clean_env[]={"PATH=/usr/bin:/bin","HOME=/","USER=faction",NULL};
#endif
static void deny(void) {
    static const char message[]="Faction isolation: startup denied.\n";
    (void)write(2,message,sizeof(message)-1);_exit(1);
}
static int protected_dir(const char *path) {
    int fd=open(path,O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC);struct stat st;
    if(fd<0 || fstat(fd,&st)<0 || st.st_uid!=0 || (st.st_mode&022))deny();
    return fd;
}
static uint32_t number(char **cursor) {
    uint32_t value=0;char *p=*cursor;unsigned count=0;
    while(*p>='0' && *p<='9') {
        unsigned digit=(unsigned)(*p++-'0');
        if(++count>10 || value>(UINT32_MAX-digit)/10)deny();
        value=value*10+digit;
    }
    if(!count || !value || *p++!='\n')deny();
    *cursor=p;return value;
}
static void noexec(const char *path,int readonly) {
    struct statvfs v;
    unsigned long flags=ST_NOEXEC|ST_NOSUID|ST_NODEV;
    if(readonly)flags|=ST_RDONLY;
    if(statvfs(path,&v)<0 || (v.f_flag&flags)!=flags)deny();
}
__attribute__((constructor(101))) static void faction_isolation_startup(void) {
    struct rlimit core={0,0};
    if(geteuid()!=0 || setrlimit(RLIMIT_CORE,&core)<0)deny();
    /* This C initializer owns startup before Rust/main can own an FD or thread. */
    if(closefrom(3)<0 && errno!=EBADF)deny();
    int fd=protected_dir("/etc");close(fd);
    fd=protected_dir("/etc/faction");close(fd);
    fd=open("/etc/faction/isolation.ids",O_RDONLY|O_NOFOLLOW|O_NONBLOCK|O_CLOEXEC);
    struct stat st;char policy[512];
    if(fd<0 || fstat(fd,&st)<0 || !S_ISREG(st.st_mode) || st.st_uid!=0 ||
       st.st_nlink!=1 || (st.st_mode&077) || st.st_size<8 || st.st_size>510)deny();
    ssize_t used=read(fd,policy,(size_t)st.st_size);close(fd);
    if(used!=st.st_size || memchr(policy,0,(size_t)used))deny();
    policy[used]=0;
    char *p=policy;uint32_t owner=number(&p),terminal=number(&p),group=number(&p);
    if(owner==terminal || (uid_t)owner!=owner || (uid_t)terminal!=terminal || (gid_t)group!=group)deny();
    char *state=p;size_t length=strlen(state);
    if(length<3 || state[0]!='/' || state[length-1]!='\n')deny();
    state[--length]=0;
    if(state[length-1]=='/' || strstr(state,"//") || strstr(state,"/.") || length>240)deny();
    for(size_t i=0;i<length;i++) {
        unsigned char c=(unsigned char)state[i];
        if(!((c>='a'&&c<='z')||(c>='A'&&c<='Z')||(c>='0'&&c<='9')||c=='/'||c=='_'||c=='-'))deny();
    }
    /* Fixed numeric MIB avoids libc name-resolution allocation per frontend. */
    int mib[]={CTL_VFS,VFS_GENERIC,VFS_USERMOUNT};
    int usermount=-1;size_t size=sizeof(usermount);
    if(sysctl(mib,3,&usermount,&size,NULL,0)<0 || size!=sizeof(usermount) || usermount!=0)deny();
    uid_t target;
#ifdef FACTION_TERMINAL
    target=(uid_t)terminal;
    if(getuid()!=target || getgid()!=(gid_t)group || getegid()!=(gid_t)group ||
       getenv("SSH_ORIGINAL_COMMAND") || !getenv("SSH_CONNECTION"))deny();
    gid_t groups[64];int ngroups=getgroups(64,groups);if(ngroups<0)deny();
    for(int i=0;i<ngroups;i++)if(groups[i]!=(gid_t)group)deny();
    struct stat a,b,c;
    if(fstat(0,&a)<0||fstat(1,&b)<0||fstat(2,&c)<0||a.st_uid!=target||
       !isatty(0)||!isatty(1)||!isatty(2)||a.st_dev!=b.st_dev||a.st_ino!=b.st_ino||a.st_rdev!=b.st_rdev||
       a.st_dev!=c.st_dev||a.st_ino!=c.st_ino||a.st_rdev!=c.st_rdev)deny();
    const char *term=getenv("TERM");if(!term)term="dumb";
    size_t n=strnlen(term,64);if(!n||n>63)deny();
    memcpy(term_env,"TERM=",5);memcpy(term_env+5,term,n);term_env[n+5]=0;
    for(size_t i=0;i<n;i++) {
        unsigned char c=(unsigned char)term[i];
        if(!((c>='a'&&c<='z')||(c>='A'&&c<='Z')||(c>='0'&&c<='9')||c=='-'||c=='_'||c=='+'||c=='.'))deny();
    }
    environ=terminal_env;
    const char *root="/var/chroot/faction";
#else
    target=(uid_t)owner;
    if(getuid()!=0)deny();
    /* Only deliberate inherited output authority: a private root-owned log. */
    struct stat a,b;
    if(fstat(1,&a)<0||fstat(2,&b)<0||!S_ISREG(a.st_mode)||a.st_uid!=0||a.st_nlink!=1||
       (a.st_mode&077)||a.st_dev!=b.st_dev||a.st_ino!=b.st_ino)deny();
    environ=clean_env;
    const char *root="/var/chroot/faction";
#endif
    fd=protected_dir("/var");close(fd);fd=protected_dir("/var/chroot");close(fd);
    fd=protected_dir(root);
    if(fchroot(fd)<0 || chdir("/")<0)deny();
    close(fd);
    noexec("/",1);noexec("/var/run/faction",0);
#ifndef FACTION_TERMINAL
    noexec(state,0);
#endif
    gid_t gid=(gid_t)group;
    if(setgroups(1,&gid)<0||setgid(gid)<0||setuid(target)<0 ||
       getuid()!=target||geteuid()!=target||getgid()!=gid||getegid()!=gid)deny();
    /* These must fail: saved privileged IDs must not survive the transition. */
    if(seteuid(0)==0 || setegid(0)==0)deny();
}
