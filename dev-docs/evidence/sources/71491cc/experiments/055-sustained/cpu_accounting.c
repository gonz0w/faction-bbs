/* Bounded CPU-accounting cross-check in the disposable NetBSD VM only. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/sysctl.h>
#include <sys/sched.h>
#include <sys/resource.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

struct reading {
    double wall,process,thread,user,system;
    uint64_t aggregate[CPUSTATES],per_cpu[2][CPUSTATES];
};
static void require(int ok) { if(!ok) exit(1); }
static double seconds(clockid_t id) {
    struct timespec t;require(!clock_gettime(id,&t));return (double)t.tv_sec+(double)t.tv_nsec/1e9;
}
static struct reading read_counters(void) {
    struct reading r;size_t n=sizeof(r.aggregate);struct rusage usage;
    require(!sysctlbyname("kern.cp_time",r.aggregate,&n,NULL,0) && n==sizeof(r.aggregate));
    for(int i=0;i<2;i++) {
        int mib[]={CTL_KERN,KERN_CP_TIME,i};n=sizeof(r.per_cpu[i]);
        require(!sysctl(mib,3,r.per_cpu[i],&n,NULL,0) && n==sizeof(r.per_cpu[i]));
    }
    require(!getrusage(RUSAGE_SELF,&usage));
    r.user=usage.ru_utime.tv_sec+usage.ru_utime.tv_usec/1e6;
    r.system=usage.ru_stime.tv_sec+usage.ru_stime.tv_usec/1e6;
    r.process=seconds(CLOCK_PROCESS_CPUTIME_ID);r.thread=seconds(CLOCK_THREAD_CPUTIME_ID);
    r.wall=seconds(CLOCK_MONOTONIC);return r;
}
static void vector(const uint64_t *p) {
    putchar('[');for(int i=0;i<CPUSTATES;i++) printf("%s%"PRIu64,i?",":"",p[i]);putchar(']');
}
static void emit(const char *mode,struct reading a,struct reading b,int comma) {
    printf("%s{\"mode\":\"%s\",\"wall_seconds\":%.9f,\"process_seconds\":%.9f,\"thread_seconds\":%.9f,\"rusage_user_seconds\":%.9f,\"rusage_system_seconds\":%.9f,\"aggregate_before\":",comma?",":"",mode,b.wall-a.wall,b.process-a.process,b.thread-a.thread,b.user-a.user,b.system-a.system);
    vector(a.aggregate);printf(",\"aggregate_after\":");vector(b.aggregate);
    printf(",\"per_cpu_before\":[");vector(a.per_cpu[0]);putchar(',');vector(a.per_cpu[1]);
    printf("],\"per_cpu_after\":[");vector(b.per_cpu[0]);putchar(',');vector(b.per_cpu[1]);printf("]}");
}
int main(void) {
    int cpus=0;size_t n=sizeof(cpus);require(!sysctlbyname("hw.ncpu",&cpus,&n,NULL,0) && cpus==2);
    require(access("/etc/faction-disposable-test",F_OK)==0);alarm(25);
    printf("{\"schema\":1,\"cpu_count\":2,\"idle_index\":%d,\"phases\":[",CP_IDLE);
    const char *modes[]={"sleep","busy","periodic_30ms_busy_20ms_sleep"};
    volatile uint64_t work=1;
    for(int mode=0;mode<3;mode++) {
        struct reading a=read_counters();double end=a.wall+5;
        while(seconds(CLOCK_MONOTONIC)<end) {
            if(mode==0) { struct timespec t={0,10000000};nanosleep(&t,NULL);continue; }
            double until=mode==2?seconds(CLOCK_MONOTONIC)+.030:end;
            while(seconds(CLOCK_MONOTONIC)<until) for(unsigned i=0;i<1024;i++) work=work*UINT64_C(6364136223846793005)+1;
            if(mode==2) { struct timespec t={0,20000000};nanosleep(&t,NULL); }
        }
        struct reading b=read_counters();emit(modes[mode],a,b,mode!=0);
    }
    printf("]}\n");return fflush(stdout)?1:0;
}
