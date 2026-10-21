/* Read-only marked native fixture audit. Never emit environment or identity. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/param.h>
#include <sys/stat.h>
#include <sys/sysctl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
static void require(int ok) { if(!ok) exit(1); }
int main(int argc,char **argv) {
    struct stat st;require(geteuid()==0 && !lstat("/etc/faction-disposable-test",&st) && S_ISREG(st.st_mode) && st.st_uid==0);
    require(argc==2);char *end;long listener=strtol(argv[1],&end,10);require(!*end && listener>1 && listener<INT_MAX);
    int mib[]={CTL_KERN,KERN_PROC2,KERN_PROC_ALL,0,sizeof(struct kinfo_proc2),INT_MAX};size_t size=0;
    require(!sysctl(mib,6,NULL,&size,NULL,0));size+=32*sizeof(struct kinfo_proc2);
    struct kinfo_proc2 *rows=malloc(size);require(rows!=NULL);mib[5]=(int)(size/sizeof(*rows));require(!sysctl(mib,6,rows,&size,NULL,0));
    size_t count=size/sizeof(*rows);unsigned checked=0;
    for(size_t i=0;i<count;i++) {
        int parent=rows[i].p_ppid,related=0;
        for(int depth=0;depth<32 && parent>1;depth++) {
            if(parent==listener) { related=1;break; }
            size_t j;for(j=0;j<count;j++) if(rows[j].p_pid==parent) { parent=rows[j].p_ppid;break; }
            if(j==count) break;
        }
        if(!related) continue;
        int query[]={CTL_KERN,KERN_PROC_ARGS,rows[i].p_pid,KERN_PROC_ENV};size_t n=0;
        require(!sysctl(query,4,NULL,&n,NULL,0));require(n<1024*1024);
        char *env=calloc(n+1,1);require(env!=NULL);require(!sysctl(query,4,env,&n,NULL,0));
        for(size_t offset=0;offset<n;) {
            require(strncmp(env+offset,"LD_PRELOAD=",11) && strncmp(env+offset,"LD_BIND_NOW=",12));
            size_t length=strnlen(env+offset,n-offset);require(length<n-offset);offset+=length+1;
        }
        free(env);checked++;
    }
    free(rows);require(checked>=3);printf("{\"checked_member_descendants\":%u,\"loader_environment_leaks\":0}\n",checked);return 0;
}
