/* Disposable calibration only: known touched anonymous allocations, no secrets. */
#define _POSIX_C_SOURCE 200809L
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>
static uint64_t now(void) {
    struct timespec t;if(clock_gettime(CLOCK_MONOTONIC,&t)) exit(1);
    return (uint64_t)t.tv_sec*1000000000+(uint64_t)t.tv_nsec;
}
int main(int argc,char **argv) {
    if(argc!=3) return 2;
    char *end;long mib=strtol(argv[1],&end,10);if(*end||mib<0||mib>64) return 2;
    long ms=strtol(argv[2],&end,10);if(*end||ms<20||ms>2000) return 2;
    size_t bytes=(size_t)mib*1024*1024;volatile unsigned char *p=malloc(bytes?bytes:1);
    if(!p) return 1;
    struct timespec pause={0,500000000};nanosleep(&pause,NULL);
    uint64_t began=now();long page=sysconf(_SC_PAGESIZE);if(page<1) return 1;
    for(size_t i=0;i<bytes;i+=(size_t)page) p[i]=1;
    uint64_t touched=now();
    pause=(struct timespec){ms/1000,(ms%1000)*1000000};
    while(nanosleep(&pause,&pause)<0 && errno==EINTR) {}
    free((void *)p);
    printf("{\"bytes\":%zu,\"began_ns\":%llu,\"touched_ns\":%llu,\"released_ns\":%llu}\n",bytes,(unsigned long long)began,(unsigned long long)touched,(unsigned long long)now());
    return 0;
}
