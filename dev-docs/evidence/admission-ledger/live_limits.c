/* Root-only, read-only live NetBSD resource attribution; no IDs in output. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/param.h>
#include <sys/resource.h>
#include <sys/sysctl.h>
#include <sys/stat.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
static void fail(void) { perror("live limits"); exit(1); }
static int number(const char *s) {
 char *end; errno=0; long n=strtol(s,&end,10);
 if(errno || !*s || *end || n<1 || n>INT_MAX) exit(2);
 return (int)n;
}
static int descendant(const struct kinfo_proc2 *p, const struct kinfo_proc2 *all, size_t n, int root) {
 int parent=p->p_ppid;
 for(int depth=0;depth<32 && parent>1;depth++) {
  if(parent==root) return 1;
  size_t i;for(i=0;i<n;i++) if(all[i].p_pid==parent) { parent=all[i].p_ppid;break; }
  if(i==n) break;
 }
 return 0;
}
static void value(rlim_t v) {
 if(v==RLIM_INFINITY) printf("\"unlimited\""); else printf("%"PRIu64,(uint64_t)v);
}
int main(int argc,char **argv) {
 struct stat marker;
 if(argc!=4 || geteuid()!=0 || lstat("/etc/faction-disposable-test",&marker)<0 ||
    !S_ISREG(marker.st_mode) || marker.st_uid!=0) return 2;
 uid_t owner=(uid_t)number(argv[1]),term=(uid_t)number(argv[2]);int listener=number(argv[3]);
 if(owner==term) return 2;
 int mib[]={CTL_KERN,KERN_PROC2,KERN_PROC_ALL,0,sizeof(struct kinfo_proc2),INT_MAX};
 struct kinfo_proc2 *all=NULL;size_t n=0,length;
 for(int attempt=0;attempt<4;attempt++) {
  length=0;if(sysctl(mib,6,NULL,&length,NULL,0)<0) fail();
  length+=32*sizeof(*all);all=malloc(length);if(!all) fail();mib[5]=(int)(length/sizeof(*all));
  if(sysctl(mib,6,all,&length,NULL,0)==0) { n=length/sizeof(*all);break; }
  free(all);all=NULL;mib[5]=INT_MAX;
 }
 if(!all) fail();
 const char *names[]={"cpu_seconds","file_bytes","data_bytes","stack_bytes","core_bytes","rss_bytes",
  "locked_bytes","processes","files","socket_bytes","address_bytes","threads"};
 _Static_assert(sizeof(names)/sizeof(names[0])==RLIM_NLIMITS,"update native resource names");
 printf("{\"resources\":[");for(int j=0;j<RLIM_NLIMITS;j++) printf("%s\"%s\"",j?",":"",names[j]);
 printf("],\"processes\":[");int comma=0;
 for(size_t i=0;i<n;i++) {
  struct kinfo_proc2 *p=&all[i];const char *role=NULL;
  if(p->p_uid==owner && !strcmp(p->p_comm,"faction-service")) role="broker";
  else if(p->p_uid==owner && !strcmp(p->p_comm,"faction")) role="owner_tool";
  else if(p->p_uid==term && !strcmp(p->p_comm,"faction-tty")) role="frontend";
  else if(p->p_pid==listener || (descendant(p,all,n,listener) && !strncmp(p->p_comm,"sshd",4)))
   role=p->p_pid==listener?"ssh_listener":(p->p_uid==0?"ssh_privileged":"ssh_session");
  if(!role) continue;
  rlim_t limits[RLIM_NLIMITS][2];
  for(int j=0;j<RLIM_NLIMITS;j++) for(int type=0;type<2;type++) {
   int q[]={CTL_PROC,p->p_pid,PROC_PID_LIMIT,j+1,type+1};length=sizeof(rlim_t);
   if(sysctl(q,5,&limits[j][type],&length,NULL,0)<0 || length!=sizeof(rlim_t)) fail();
  }
  printf("%s{\"role\":\"%s\",\"rss_bytes\":%"PRIu64",\"virtual_bytes\":%"PRIu64",\"limits\":[",
   comma++?",":"",role,(uint64_t)p->p_vm_rssize*(uint64_t)getpagesize(),(uint64_t)p->p_vm_msize*(uint64_t)getpagesize());
  for(int j=0;j<RLIM_NLIMITS;j++) { printf("%s[",j?",":"");value(limits[j][0]);printf(",");value(limits[j][1]);printf("]"); }
  printf("]}");
 }
 printf("]}\n");free(all);return fflush(stdout)?1:0;
}
