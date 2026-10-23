/* BSD-3-Clause. Narrow kernel process/peer boundary; no domain authority.
 * Call prepare once, before starting threads/opening board files. Limits are
 * irreversible reductions, not filesystem/network syscall filtering.
 */
#define _NETBSD_SOURCE 1
#define _GNU_SOURCE 1
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/resource.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>

static int ceiling(int resource, rlim_t value) {
    struct rlimit previous, next = { value, value }, actual;
    if (getrlimit(resource, &previous) < 0 || previous.rlim_cur < value || previous.rlim_max < value) {
        errno = EPERM; return -1;
    }
    if (setrlimit(resource, &next) < 0 || getrlimit(resource, &actual) < 0) return -1;
    if (actual.rlim_cur != value || actual.rlim_max != value) { errno = EPERM; return -1; }
    return 0;
}

int faction_prepare_process(void) {
    gid_t groups[64];
    int n;
    if (getuid() == 0 || getuid() != geteuid() || getgid() == 0 || getgid() != getegid()) {
        errno = EPERM; return -1;
    }
    n = getgroups(64, groups);
    if (n < 0) return -1;
    for (int i = 0; i < n; i++) if (groups[i] != getgid()) { errno = EPERM; return -1; }
    if (ceiling(RLIMIT_CORE, 0) < 0 || ceiling(RLIMIT_MEMLOCK, 0) < 0) return -1;
    /* Reject unknown inherited descriptors; never close Rust-owned File/OwnedFd
     * objects through a safe API. The native C frontend closes its own inherited
     * descriptors before entering this shared check. */
#ifdef __NetBSD__
    int maximum = fcntl(0, F_MAXFD);
    if (maximum != 2) { errno = EPERM; return -1; }
#endif
    for (int fd = 0; fd < 3; fd++) {
        int flags = fcntl(fd, F_GETFD);
        if (flags < 0 || fcntl(fd, F_SETFD, flags | FD_CLOEXEC) < 0) return -1;
    }
    if (chdir("/") < 0) return -1;
    return 0;
}

int faction_process_limits(int service, unsigned sessions) {
    if ((service != 0 && service != 1) || (service && (sessions < 1 || sessions > 250))) {
        errno = EINVAL; return -1;
    }
    rlim_t files = service ? (rlim_t)sessions * 2 + 48 : 8;
    if (ceiling(RLIMIT_NOFILE, files) < 0 || ceiling(RLIMIT_NPROC, 0) < 0) return -1;
#ifdef __NetBSD__
    /* NetBSD counts extra LWPs separately from the first thread of each process.
     * Service has one handoff and two fixed credential threads before this call. */
    if (ceiling(RLIMIT_NTHR, service ? 3 : 0) < 0) return -1;
#endif
    return 0;
}

int faction_peer_uid(int socket_fd, uint32_t *uid) {
    if (uid == NULL) { errno = EINVAL; return -1; }
#ifdef __linux__
    struct ucred credential;
    socklen_t length = sizeof(credential);
    if (getsockopt(socket_fd, SOL_SOCKET, SO_PEERCRED, &credential, &length) < 0) return -1;
    if (length != sizeof(credential)) { errno = EINVAL; return -1; }
    *uid = (uint32_t)credential.uid;
#else
    uid_t user;
    gid_t group;
    if (getpeereid(socket_fd, &user, &group) < 0) return -1;
    *uid = (uint32_t)user;
#endif
    return 0;
}
