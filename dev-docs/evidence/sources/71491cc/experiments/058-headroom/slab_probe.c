/* Effective native allocator size classes; no state or configuration writes. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
extern int __je_mallctl(const char *,void *,size_t *,void *,size_t);
static void read_value(const char *name,void *value,size_t size) {
 size_t n=size;if(__je_mallctl(name,value,&n,NULL,0) || n!=size) exit(1);
}
int main(void) {
 unsigned bins;read_value("arenas.nbins",&bins,sizeof(bins));if(bins<1 || bins>128) return 1;
 printf("{\"page_bytes\":%d,\"columns\":[\"size_bytes\",\"regions\",\"slab_bytes\",\"shards\"],\"bins\":[",getpagesize());
 for(unsigned i=0;i<bins;i++) {
  char name[80];size_t size,slab;uint32_t regions,shards;
  snprintf(name,sizeof(name),"arenas.bin.%u.size",i);read_value(name,&size,sizeof(size));
  snprintf(name,sizeof(name),"arenas.bin.%u.slab_size",i);read_value(name,&slab,sizeof(slab));
  snprintf(name,sizeof(name),"arenas.bin.%u.nregs",i);read_value(name,&regions,sizeof(regions));
  snprintf(name,sizeof(name),"arenas.bin.%u.nshards",i);read_value(name,&shards,sizeof(shards));
  printf("%s[%zu,%"PRIu32",%zu,%"PRIu32"]",i?",":"",size,regions,slab,shards);
 }
 puts("]}");return fflush(stdout)?1:0;
}
