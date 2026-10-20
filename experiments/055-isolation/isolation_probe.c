/* BSD-3-Clause. Marked private NetBSD fixture only; not a shipped entry point. */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <unistd.h>
#include <fcntl.h>
#include <grp.h>
#include <poll.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern int faction_prepare_process(void);
extern int faction_process_limits(int,unsigned);
extern char **environ;
#define CHECK(x) do{if(!(x)){dprintf(2,"probe failed line %d errno %d\n",__LINE__,errno);return 1;}}while(0)
static unsigned port(const char *s){char *end;unsigned long n=strtoul(s,&end,10);return *s&&!*end&&n>0&&n<65536?(unsigned)n:0;}
static int address(int family,unsigned port,struct sockaddr_storage *storage){
 memset(storage,0,sizeof(*storage));
 if(family==AF_INET){struct sockaddr_in *a=(void *)storage;a->sin_len=sizeof(*a);a->sin_family=family;a->sin_port=htons(port);a->sin_addr.s_addr=htonl(INADDR_LOOPBACK);return sizeof(*a);}
 struct sockaddr_in6 *a=(void *)storage;a->sin6_len=sizeof(*a);a->sin6_family=family;a->sin6_port=htons(port);a->sin6_addr=in6addr_loopback;return sizeof(*a);
}
int main(int argc,char **argv){
 CHECK(argc==5);for(int i=1;i<5;i++)CHECK(port(argv[i]));
 CHECK(faction_prepare_process()==0);
#ifdef FACTION_TERMINAL
 CHECK(faction_process_limits(0,0)==0);
 const char *role="terminal";
#else
 CHECK(faction_process_limits(1,250)==0);
 const char *role="owner";
#endif
 CHECK(getuid()!=0&&getuid()==geteuid()&&getgid()!=0&&getgid()==getegid());
 CHECK(seteuid(0)==-1&&setegid(0)==-1);
 CHECK(setgroups(0,NULL)==-1&&setgid((gid_t)32766)==-1);
 CHECK(open("/etc/passwd",O_RDONLY)==-1&&open("/../../etc/master.passwd",O_RDONLY)==-1);
 CHECK(chdir("/")==0&&chdir("..") ==0);char cwd[8];CHECK(getcwd(cwd,sizeof(cwd))&&strcmp(cwd,"/")==0);
 CHECK(open("/escape-created",O_WRONLY|O_CREAT,0600)==-1);
 int root=open("/",O_RDONLY|O_DIRECTORY);CHECK(root>=0);CHECK(fchroot(root)==-1);close(root);
 CHECK(open("/etc/faction/service.toml",O_WRONLY)==-1);
#ifdef FACTION_TERMINAL
 CHECK(open("/etc/faction/service.toml",O_RDONLY)==-1);
 CHECK(open("/var/db/faction-profile/faction.sqlite3",O_RDONLY)==-1);
#else
 int config=open("/etc/faction/service.toml",O_RDONLY);CHECK(config>=0);close(config);
#endif
 int executable=open("/probe-executable",O_RDONLY);CHECK(executable>=0);
 char *v[]={"probe-executable",NULL};CHECK(execve("/probe-executable",v,environ)==-1);CHECK(fexecve(executable,v,environ)==-1);close(executable);
 CHECK(execve("/bin/sh",v,environ)==-1);
 for(int index=0;index<2;index++){
  int family=index?AF_INET6:AF_INET;struct sockaddr_storage peer;int length=address(family,port(argv[index+1]),&peer);
  int raw=socket(family,SOCK_RAW,index?IPPROTO_ICMPV6:IPPROTO_ICMP);CHECK(raw==-1);
  int datagram_icmp=socket(family,SOCK_DGRAM,index?IPPROTO_ICMPV6:IPPROTO_ICMP);CHECK(datagram_icmp==-1);
  int tcp=socket(family,SOCK_STREAM|SOCK_NONBLOCK,0);CHECK(tcp>=0);
  int rc=connect(tcp,(struct sockaddr *)&peer,(socklen_t)length);CHECK(rc==-1);
  struct pollfd wait={.fd=tcp,.events=POLLOUT};rc=poll(&wait,1,400);
  if(rc>0){int error=0;socklen_t size=sizeof(error);CHECK(getsockopt(tcp,SOL_SOCKET,SO_ERROR,&error,&size)==0);
   if(error==0){struct sockaddr_storage connected;socklen_t n=sizeof(connected);int peer_rc=getpeername(tcp,(struct sockaddr *)&connected,&n);CHECK(peer_rc==-1&&errno==ENOTCONN);}}
  close(tcp);
  int udp=socket(family,SOCK_DGRAM|SOCK_NONBLOCK,0);CHECK(udp>=0);
  ssize_t sent=sendto(udp,"blocked",7,0,(struct sockaddr *)&peer,(socklen_t)length);
  CHECK(sent==7 || (sent==-1 && (errno==EPERM||errno==EACCES||errno==EHOSTUNREACH||errno==ENETUNREACH)));
  wait.fd=udp;wait.events=POLLIN;CHECK(poll(&wait,1,250)==0);close(udp);
  /* Unsolicited receive: parent sends repeated root-owned packets to this port. */
  length=address(family,port(argv[index+3]),&peer);udp=socket(family,SOCK_DGRAM|SOCK_NONBLOCK,0);CHECK(udp>=0);
  CHECK(bind(udp,(struct sockaddr *)&peer,(socklen_t)length)==0);
  wait.fd=udp;CHECK(poll(&wait,1,400)==0);close(udp);
 }
 printf("{\"role\":\"%s\",\"privilege_drop\":true,\"filesystem\":true,\"exec\":true,\"tcp_udp_v4_v6\":true,\"raw_socket_denied\":true}\n",role);
 return 0;
}
