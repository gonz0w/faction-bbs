/* Unprivileged startup detachment only. No resident supervisor or setuid bit. */
#define _NETBSD_SOURCE
#include <err.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int
main(int argc, char **argv)
{
    char *const environment[] = {
        "PATH=/usr/bin:/bin", "HOME=/home/factiond",
        "USER=factiond", "LOGNAME=factiond", NULL
    };
    if (argc < 2 || getuid() == 0 || getuid() != geteuid() || getgid() != getegid())
        errx(1, "requires an unprivileged service owner");
    if (strcmp(argv[1], "/usr/local/libexec/faction-service") != 0 &&
        strcmp(argv[1], "/usr/local/libexec/faction-broker") != 0)
        errx(1, "requires a fixed Faction backend");
    closefrom(3);
    /* Keep the private rc log stdio; leave the boot controlling session. */
    if (daemon(0, 1) == -1)
        err(1, "detach");
    execve(argv[1], argv + 1, environment);
    err(1, "exec backend");
}
