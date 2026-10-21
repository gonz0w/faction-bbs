/* Read-only native fixture assertion: the gate must map in the listener only. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/param.h>
#include <sys/stat.h>
#include <sys/sysctl.h>
#include <uvm/uvm_param.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
static void require(int ok) { if(!ok) exit(1); }
static unsigned maps(int pid) {
    int mib[]={CTL_VM,VM_PROC,VM_PROC_MAP,pid,sizeof(struct kinfo_vmentry)};
    size_t size=0;require(!sysctl(mib,5,NULL,&size,NULL,0));
    require(size<16*1024*1024);size+=32*sizeof(struct kinfo_vmentry);
    struct kinfo_vmentry *rows=malloc(size);require(rows!=NULL);
    require(!sysctl(mib,5,rows,&size,NULL,0));unsigned found=0;
    for(size_t i=0;i<size/sizeof(*rows);i++)
        if(strstr(rows[i].kve_path,"faction-ssh-gate.so")) found++;
    free(rows);return found;
}
int main(int argc,char **argv) {
    struct stat st;require(geteuid()==0 && !lstat("/etc/faction-disposable-test",&st) && S_ISREG(st.st_mode) && st.st_uid==0);
    require(argc==2);char *end;long listener=strtol(argv[1],&end,10);require(!*end && listener>1 && listener<INT_MAX);
    int mib[]={CTL_KERN,KERN_PROC2,KERN_PROC_ALL,0,sizeof(struct kinfo_proc2),INT_MAX};
    size_t size=0;require(!sysctl(mib,6,NULL,&size,NULL,0));size+=32*sizeof(struct kinfo_proc2);
    struct kinfo_proc2 *rows=malloc(size);require(rows!=NULL);mib[5]=(int)(size/sizeof(*rows));
    require(!sysctl(mib,6,rows,&size,NULL,0));unsigned checked=0,leaked=0,listener_maps=0;
    for(size_t i=0;i<size/sizeof(*rows);i++) {
        struct kinfo_proc2 *p=&rows[i];
        if(p->p_pid==listener) listener_maps=maps(p->p_pid);
        else if(!strcmp(p->p_comm,"sshd-session") || !strcmp(p->p_comm,"sshd-auth") || !strcmp(p->p_comm,"faction-tty")) {
            checked++;if(maps(p->p_pid)) leaked++;
        }
    }
    free(rows);require(listener_maps && checked && !leaked);
    printf("{\"listener_mappings\":%u,\"checked_session_auth_frontend_processes\":%u,\"leaked_processes\":%u}\n",listener_maps,checked,leaked);
    return 0;
}
