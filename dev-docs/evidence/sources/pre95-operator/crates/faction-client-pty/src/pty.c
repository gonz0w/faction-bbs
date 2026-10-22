/* BSD-3-Clause. Client-only PTY allocation, never linked by the BBS owner. */
#define _NETBSD_SOURCE 1
#define _DEFAULT_SOURCE 1
#include <sys/types.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#ifdef __linux__
#include <pty.h>
#else
#include <util.h>
#endif
int faction_client_openpty(int *master, int *slave, unsigned short rows, unsigned short cols) {
    if (master == NULL || slave == NULL || rows < 18 || rows > 100 || cols < 40 || cols > 240) {
        errno = EINVAL; return -1;
    }
    int m = -1, s = -1;
    struct winsize w = { .ws_row = rows, .ws_col = cols, .ws_xpixel = 0, .ws_ypixel = 0 };
    if (openpty(&m, &s, NULL, NULL, &w) < 0) return -1;
    if (fcntl(m, F_SETFD, FD_CLOEXEC) < 0 || fcntl(s, F_SETFD, FD_CLOEXEC) < 0) {
        int saved = errno; close(m); close(s); errno = saved; return -1;
    }
    *master = m; *slave = s; return 0;
}
