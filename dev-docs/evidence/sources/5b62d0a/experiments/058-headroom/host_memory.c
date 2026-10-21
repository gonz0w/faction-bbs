/* Read-only fixture host-service attribution; no PIDs, paths or addresses. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/param.h>
#include <sys/sysctl.h>
#include <sys/stat.h>
#include <uvm/uvm_param.h>
#include <inttypes.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
static void fail(void) { perror("host memory");exit(1); }
static uint64_t locked_ranges(int pid) {
 int mib[]={CTL_VM,VM_PROC,VM_PROC_MAP,pid,sizeof(struct kinfo_vmentry)};
 size_t length=0;if(sysctl(mib,5,NULL,&length,NULL,0)<0) fail();
 if(length>16*1024*1024) exit(1);
 length+=16*sizeof(struct kinfo_vmentry);struct kinfo_vmentry *maps=malloc(length);if(!maps) fail();
 if(sysctl(mib,5,maps,&length,NULL,0)<0) fail();
 uint64_t result=0;
 for(size_t i=0;i<length/sizeof(*maps);i++) if(maps[i].kve_wired_count) result+=maps[i].kve_end-maps[i].kve_start;
 free(maps);return result;
}
int main(void) {
 struct stat marker;if(geteuid()!=0 || lstat("/etc/faction-disposable-test",&marker)<0 || !S_ISREG(marker.st_mode) || marker.st_uid!=0) return 2;
 int mib[]={CTL_KERN,KERN_PROC2,KERN_PROC_ALL,0,sizeof(struct kinfo_proc2),INT_MAX};
 size_t length=0;if(sysctl(mib,6,NULL,&length,NULL,0)<0) fail();length+=32*sizeof(struct kinfo_proc2);
 struct kinfo_proc2 *all=malloc(length);if(!all) fail();mib[5]=(int)(length/sizeof(*all));
 if(sysctl(mib,6,all,&length,NULL,0)<0) fail();
 const char *names[]={"ntpd","master","qmgr","pickup","mdnsd","inetd","syslogd","dhcpcd","cron"};
 printf("{\"columns\":[\"service\",\"rss_bytes\",\"virtual_bytes\",\"locked_mapping_virtual_bytes\"],\"processes\":[");int comma=0;
 for(size_t i=0;i<length/sizeof(*all);i++) for(size_t j=0;j<sizeof(names)/sizeof(names[0]);j++) if(!strcmp(all[i].p_comm,names[j])) {
  uint64_t locked=!strcmp(names[j],"ntpd")?locked_ranges(all[i].p_pid):0;
  printf("%s[\"%s\",%"PRIu64",%"PRIu64",%"PRIu64"]",comma++?",":"",names[j],
   (uint64_t)all[i].p_vm_rssize*(uint64_t)getpagesize(),(uint64_t)all[i].p_vm_msize*(uint64_t)getpagesize(),locked);
 }
 puts("]}");free(all);return fflush(stdout)?1:0;
}
