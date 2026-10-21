/* Native DSS preference readback; no configuration changes. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <stdio.h>
#include <string.h>
extern int __je_mallctl(const char *,void *,size_t *,void *,size_t);
int main(void) {
 const char *dss;size_t n=sizeof(dss);
 if(__je_mallctl("opt.dss",&dss,&n,NULL,0) || n!=sizeof(dss)) return 1;
 if(strcmp(dss,"primary") && strcmp(dss,"secondary") && strcmp(dss,"disabled")) return 1;
 printf("{\"dss\":\"%s\"}\n",dss);return fflush(stdout)?1:0;
}
