/* BSD-3-Clause. Disposable native process-envelope oracles, not an installed tool. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/resource.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <errno.h>
#include <dirent.h>
#include <fcntl.h>
#include <grp.h>
#include <pthread.h>
#include <pwd.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
extern int faction_prepare_process(void);
extern int faction_process_limits(int, unsigned);
static pthread_mutex_t mutex=PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t condition=PTHREAD_COND_INITIALIZER;
static int ready_count,done;
static void *waiter(void *unused) {
    (void)unused;pthread_mutex_lock(&mutex);ready_count++;pthread_cond_broadcast(&condition);
    while(!done)pthread_cond_wait(&condition,&mutex);
    pthread_mutex_unlock(&mutex);return NULL;
}
static int check(int mode) {
    if(closefrom(3)<0 && errno!=EBADF)return 1;
    if(faction_prepare_process()<0) { perror("prepare"); return 1; }
    pthread_t threads[3];
    if(mode) {
        for(int i=0;i<3;i++)if(pthread_create(&threads[i],NULL,waiter,NULL))return 1;
        pthread_mutex_lock(&mutex);while(ready_count!=3)pthread_cond_wait(&condition,&mutex);pthread_mutex_unlock(&mutex);
    }
    if(faction_process_limits(mode,250)<0) { perror("limits");return 1; }
    int resources[]={RLIMIT_CORE,RLIMIT_MEMLOCK,RLIMIT_NPROC,RLIMIT_NTHR,RLIMIT_NOFILE};
    rlim_t expected[]={0,0,0,mode?3:0,mode?548:8};
    for(size_t i=0;i<sizeof(resources)/sizeof(resources[0]);i++) {
        struct rlimit value;
        if(getrlimit(resources[i],&value)<0 || value.rlim_cur!=expected[i] || value.rlim_max!=expected[i])return 1;
        value.rlim_cur++;value.rlim_max++;
        if(setrlimit(resources[i],&value)==0)return 1;
    }
    errno=0;pid_t child=fork();
    if(child==0)_exit(1);
    if(child>0) { (void)waitpid(child,NULL,0);return 1; }
    if(errno!=EAGAIN)return 1;
    pthread_t extra;
    if(pthread_create(&extra,NULL,waiter,NULL)==0)return 1;
    void *page=mmap(NULL,4096,PROT_READ|PROT_WRITE,MAP_ANON|MAP_PRIVATE,-1,0);
    if(page==MAP_FAILED || mlock(page,4096)==0)return 1;
    munmap(page,4096);
    int descriptors[600],count=0;
    while(count<600) {
        int fd=open("/dev/null",O_RDONLY);
        if(fd<0)break;
        descriptors[count++]=fd;
    }
    if(errno!=EMFILE || count!=(mode?545:5))return 1;
    for(int i=0;i<count;i++)close(descriptors[i]);
    if(mode) {
        pthread_mutex_lock(&mutex);done=1;pthread_cond_broadcast(&condition);pthread_mutex_unlock(&mutex);
        for(int i=0;i<3;i++)if(pthread_join(threads[i],NULL))return 1;
    }
    return 0;
}
int main(void) {
    struct stat marker;
    if(geteuid()!=0 || lstat("/etc/faction-disposable-test",&marker)<0 || !S_ISREG(marker.st_mode) || marker.st_uid!=0)return 2;
    for(int mode=0;mode<2;mode++) {
        struct passwd *p=getpwnam(mode?"factiond":"factionterm");if(!p)return 2;
        uid_t uid=p->pw_uid;gid_t gid=p->pw_gid;
        pid_t child=fork();if(child<0)return 2;
        if(child==0) {
            if(setgroups(1,&gid)<0 || setgid(gid)<0 || setuid(uid)<0)_exit(2);
            _exit(check(mode));
        }
        int status;if(waitpid(child,&status,0)<0 || !WIFEXITED(status) || WEXITSTATUS(status)!=0)return 1;
    }
    struct passwd *account=getpwnam("factionterm");if(!account)return 2;
    uid_t uid=account->pw_uid;gid_t gid=account->pw_gid;
    char directory[]="/var/tmp/faction-core-XXXXXX";
    if(!mkdtemp(directory) || chown(directory,uid,gid)<0 || chmod(directory,0700)<0)return 2;
    pid_t crash=fork();if(crash<0)return 2;
    if(crash==0) {
        if(setgroups(1,&gid)<0 || setgid(gid)<0 || setuid(uid)<0)_exit(2);
        if(closefrom(3)<0 && errno!=EBADF)_exit(2);
        if(faction_prepare_process()<0 || faction_process_limits(0,0)<0 || chdir(directory)<0)_exit(2);
        abort();
    }
    int status;
    if(waitpid(crash,&status,0)<0 || !WIFSIGNALED(status) || WTERMSIG(status)!=SIGABRT || WCOREDUMP(status))return 1;
    DIR *listing=opendir(directory);if(!listing)return 2;
    struct dirent *entry;
    while((entry=readdir(listing))!=NULL)if(strcmp(entry->d_name,".") && strcmp(entry->d_name,".."))return 1;
    closedir(listing);if(rmdir(directory)<0)return 2;
    puts("{\"frontend\":\"pass\",\"service\":\"pass\",\"fork_denied\":true,\"extra_threads_denied\":true,\"mlock_denied\":true,\"descriptor_cap_enforced\":true,\"hard_limit_raise_denied\":true,\"crash_without_core_file\":true}");
    return 0;
}
