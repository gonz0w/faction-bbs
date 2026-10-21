/* NetBSD libc allocator readback only: no configuration changes or identities. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
extern int __je_mallctl(const char *, void *, size_t *, void *, size_t);
static void read_option(const char *name, void *value, size_t size) {
    size_t n=size;
    if(__je_mallctl(name,value,&n,NULL,0) || n!=size) exit(1);
}
int main(void) {
    const char *version;unsigned arenas;bool tcache,retain,stats;size_t fit,oversize;ssize_t dirty,muzzy;
    read_option("version",&version,sizeof(version));
    read_option("opt.narenas",&arenas,sizeof(arenas));
    read_option("opt.tcache",&tcache,sizeof(tcache));
    read_option("opt.retain",&retain,sizeof(retain));
    read_option("opt.dirty_decay_ms",&dirty,sizeof(dirty));
    read_option("opt.muzzy_decay_ms",&muzzy,sizeof(muzzy));
    read_option("config.stats",&stats,sizeof(stats));
    read_option("opt.lg_extent_max_active_fit",&fit,sizeof(fit));
    read_option("opt.oversize_threshold",&oversize,sizeof(oversize));
    printf("{\"config_stats\":%s,\"lg_extent_max_active_fit\":%zu,\"oversize_threshold\":%zu,",stats?"true":"false",fit,oversize);
    printf("\"version\":\"%s\",\"narenas\":%u,\"tcache\":%s,\"retain\":%s,\"dirty_decay_ms\":%zd,\"muzzy_decay_ms\":%zd,\"issetugid\":%d}\n",
       version,arenas,tcache?"true":"false",retain?"true":"false",dirty,muzzy,issetugid());
    return 0;
}
