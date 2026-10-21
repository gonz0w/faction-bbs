/* Read-only NetBSD profiling helper. No paths, identities or PIDs in JSON. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/param.h>
#include <sys/sysctl.h>
#include <sys/sched.h>
#include <uvm/uvm_extern.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static void fail(void) { perror("NetBSD metrics"); exit(1); }
static long number(const char *s) {
    char *end; errno=0; long n=strtol(s,&end,10);
    if(errno || *end || n<1 || n>INT_MAX) { exit(2); }
    return n;
}
static int open_files(void) {
    int mib[]={CTL_KERN,KERN_FILE2,KERN_FILE_BYFILE,0,sizeof(struct kinfo_file),INT_MAX};
    for(int attempt=0;attempt<4;attempt++) {
        size_t length=0;
        if(sysctl(mib,6,NULL,&length,NULL,0)<0) fail();
        length+=32*sizeof(struct kinfo_file);
        void *buffer=malloc(length);if(!buffer) fail();
        mib[5]=(int)(length/sizeof(struct kinfo_file));
        int rc=sysctl(mib,6,buffer,&length,NULL,0);free(buffer);
        if(rc==0) return (int)(length/sizeof(struct kinfo_file));
        mib[5]=INT_MAX;
    }
    fail();return 0;
}
static int descendant(const struct kinfo_proc2 *p, const struct kinfo_proc2 *all, size_t n, int root) {
    int parent=p->p_ppid;
    for (int depth=0; depth<32 && parent>1; depth++) {
        if(parent==root) return 1;
        size_t i;for(i=0;i<n;i++) if(all[i].p_pid==parent) { parent=all[i].p_ppid; break; }
        if(i==n) break;
    }
    return 0;
}
static void snapshot(uid_t owner, uid_t term, int listener) {
    struct uvmexp_sysctl vm;size_t length=sizeof(vm); int vmmib[]={CTL_VM,VM_UVMEXP2};
    if(sysctl(vmmib,2,&vm,&length,NULL,0)<0 || length!=sizeof(vm)) fail();
    uint64_t ticks[CPUSTATES]; length=sizeof(ticks);
    if(sysctlbyname("kern.cp_time",ticks,&length,NULL,0)<0 || length!=sizeof(ticks)) fail();
    int maxfiles,maxproc,nfiles;
    length=sizeof(maxfiles); if(sysctlbyname("kern.maxfiles",&maxfiles,&length,NULL,0)<0) fail();
    length=sizeof(maxproc); if(sysctlbyname("kern.maxproc",&maxproc,&length,NULL,0)<0) fail();
    nfiles=open_files();
    int mib[]={CTL_KERN,KERN_PROC2,KERN_PROC_ALL,0,sizeof(struct kinfo_proc2),INT_MAX};
    struct kinfo_proc2 *all=NULL;size_t n=0;
    for(int attempt=0;attempt<4;attempt++) {
        length=0; if(sysctl(mib,6,NULL,&length,NULL,0)<0) fail();
        length+=32*sizeof(*all);all=malloc(length);if(!all) fail();
        mib[5]=(int)(length/sizeof(*all));
        if(sysctl(mib,6,all,&length,NULL,0)==0) { n=length/sizeof(*all);break; }
        free(all);all=NULL;mib[5]=INT_MAX;
    }
    if(!all) fail();
    printf("{\"vm\":{\"page_bytes\":%"PRId64",\"total_pages\":%"PRId64",\"free_pages\":%"PRId64
      ",\"active_pages\":%"PRId64",\"inactive_pages\":%"PRId64",\"wired_pages\":%"PRId64
      ",\"anon_pages\":%"PRId64",\"file_pages\":%"PRId64",\"exec_pages\":%"PRId64
      ",\"kernel_pool_pages\":%"PRId64",\"boot_kernel_pages\":%"PRId64",\"file_clean_pages\":%"PRId64",\"file_dirty_pages\":%"PRId64",\"swap_used_pages\":%"PRId64",\"swapout_pages\":%"PRId64"},",
      vm.pagesize,vm.npages,vm.free,vm.active,vm.inactive,vm.wired,vm.anonpages,vm.filepages,vm.execpages,vm.poolpages,vm.bootpages,vm.fileclean,vm.filedirty,vm.swpginuse,vm.pgswapout);
    printf("\"cpu_ticks\":[");for(int i=0;i<CPUSTATES;i++) printf("%s%"PRIu64,i?",":"",ticks[i]);
    printf("],\"cpu_idle_index\":%d,\"limits\":{\"max_files\":%d,\"open_files\":%d,\"max_processes\":%d,\"processes\":%zu},\"processes\":[",CP_IDLE,maxfiles,nfiles,maxproc,n);
    int comma=0;
    for(size_t i=0;i<n;i++) {
        const struct kinfo_proc2 *p=&all[i];const char *role=NULL;
        if(p->p_uid==owner && !strcmp(p->p_comm,"faction-member")) role="member";
        else if(p->p_uid==owner && (!strcmp(p->p_comm,"faction-broker") || !strcmp(p->p_comm,"faction-service"))) role="broker";
        else if(p->p_uid==term && !strcmp(p->p_comm,"faction-tty")) role="frontend";
        else if(p->p_pid==listener || descendant(p,all,n,listener)) {
            role="ssh"; /* Includes transient key-command and shell setup descendants. */
        }
        if(!role) continue;
        printf("%s{\"role\":\"%s\",\"rss_kib\":%"PRId64",\"vsz_bytes\":%"PRId64
          ",\"high_water_rss_kib\":%"PRIu64",\"cpu_us\":%"PRIu64"}",comma++?",":"",role,
          (int64_t)p->p_vm_rssize*vm.pagesize/1024,(int64_t)p->p_vm_msize*vm.pagesize,
          p->p_uru_maxrss,(uint64_t)p->p_uutime_sec*1000000+p->p_uutime_usec+(uint64_t)p->p_ustime_sec*1000000+p->p_ustime_usec);
    }
    printf("]}");free(all);
}

static uint64_t monotonic_ns(void) {
    struct timespec ts; if(clock_gettime(CLOCK_MONOTONIC,&ts)<0) fail();
    return (uint64_t)ts.tv_sec*1000000000+(uint64_t)ts.tv_nsec;
}
int main(int argc, char **argv) {
    if(argc!=4 && argc!=7) return 2;
    uid_t owner=(uid_t)number(argv[1]),term=(uid_t)number(argv[2]);int listener=(int)number(argv[3]);
    if(argc==4) { snapshot(owner,term,listener);puts("");return 0; }
    if(strcmp(argv[4],"--stream")) return 2;
    long ms=number(argv[5]),count=number(argv[6]);
    if(ms<5 || ms>1000 || count>12000) return 2;
    uint64_t start=monotonic_ns(),step=(uint64_t)ms*1000000;
    printf("{\"event\":\"start\",\"monotonic_ns\":%"PRIu64",\"interval_ms\":%ld,\"count\":%ld}\n",start,ms,count);fflush(stdout);
    for(long i=0;i<count;i++) {
        uint64_t began=monotonic_ns();
        printf("{\"event\":\"sample\",\"index\":%ld,\"monotonic_ns\":%"PRIu64",\"metrics\":",i,began);
        snapshot(owner,term,listener);
        printf(",\"finished_ns\":%"PRIu64"}\n",monotonic_ns());
        if(fflush(stdout)) fail();
        /* Skip missed deadlines, never emit a catch-up burst. */
        uint64_t now=monotonic_ns(),target=start+(((now-start)/step)+1)*step;
        struct timespec deadline={(time_t)(target/1000000000),(long)(target%1000000000)};
        int rc;while((rc=clock_nanosleep(CLOCK_MONOTONIC,TIMER_ABSTIME,&deadline,NULL))==EINTR) {}
        if(rc) { errno=rc;fail(); }
    }
    printf("{\"event\":\"stop\",\"monotonic_ns\":%"PRIu64"}\n",monotonic_ns());
    return fflush(stdout)?1:0;
}
