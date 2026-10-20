/* BSD-3-Clause. NetBSD terminal-only SSH entry point.
 * Reviewed FBSTTY01 descriptor handoff: no DB, crypto, shell or member authority.
 * See docs/shared-service.md for installation and the explicit native boundary.
 */
#define _NETBSD_SOURCE 1
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

extern ssize_t faction_send_fd(int, int, const unsigned char *, size_t);

static int64_t now_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) < 0) return -1;
    return (int64_t)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
}

/* One absolute deadline across retries; HUP/ERR wake the following IO too. */
static int ready(int fd, short events, int64_t deadline) {
    for (;;) {
        int64_t now = now_ms();
        if (now < 0 || now >= deadline) return -1;
        int64_t remaining = deadline - now;
        struct pollfd p = { .fd = fd, .events = events };
        int rc = poll(&p, 1, remaining > INT_MAX ? INT_MAX : (int)remaining);
        if (rc > 0) return (p.revents & POLLNVAL) ? -1 : 0;
        if (rc < 0 && errno != EINTR) return -1;
    }
}

static int owner_uid(const char *text, uid_t *owner) {
    uint32_t n = 0;
    if (!text[0]) return -1;
    for (size_t i = 0; text[i]; i++) {
        if (i >= 10 || text[i] < '0' || text[i] > '9') return -1;
        unsigned digit = (unsigned)(text[i] - '0');
        if (n > (UINT32_MAX - digit) / 10) return -1;
        n = n * 10 + digit;
    }
    if (n == 0 || n == geteuid() || (uid_t)n != n) return -1;
    *owner = (uid_t)n;
    return 0;
}

static int run(int argc, char **argv) {
    int tty = -1, control = -1, saved = 0, result = -1;
    struct termios original;
    struct stat st;
    struct sockaddr_un address;
    char parent[sizeof(address.sun_path)];
    unsigned char packet[72] = {0};
    uid_t owner;
    if (argc != 5 || strcmp(argv[1], "--socket") || strcmp(argv[3], "--owner-uid") ||
        getenv("SSH_ORIGINAL_COMMAND") != NULL || getenv("SSH_CONNECTION") == NULL ||
        !isatty(STDIN_FILENO) || !isatty(STDOUT_FILENO) || owner_uid(argv[4], &owner) < 0)
        goto done;
    size_t length = strnlen(argv[2], sizeof(address.sun_path));
    if (!length || length >= sizeof(address.sun_path)) goto done;
    const char *slash = strrchr(argv[2], '/');
    if (!slash) goto done;
    size_t prefix = (size_t)(slash - argv[2]);
    if (prefix == 0) prefix = 1;
    memcpy(parent, argv[2], prefix);
    parent[prefix] = '\0';
    if (lstat(parent, &st) < 0 || !S_ISDIR(st.st_mode) || st.st_uid != owner || (st.st_mode & 027))
        goto done;
    if (lstat(argv[2], &st) < 0 || !S_ISSOCK(st.st_mode) || st.st_uid != owner || (st.st_mode & 007))
        goto done;
    const char *term = getenv("TERM");
    if (!term) term = "dumb";
    size_t term_length = strnlen(term, 64);
    if (term_length == 0 || term_length > 63) goto done;
    for (size_t i = 0; i < term_length; i++) {
        unsigned char c = (unsigned char)term[i];
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
              (c >= '0' && c <= '9') || c == '-' || c == '_' || c == '+' || c == '.'))
            goto done;
    }
    tty = fcntl(STDIN_FILENO, F_DUPFD_CLOEXEC, 3);
    if (tty < 0 || tcgetattr(tty, &original) < 0) goto done;
    saved = 1;
    control = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0);
    if (control < 0) goto done;
    memset(&address, 0, sizeof(address));
    address.sun_family = AF_UNIX;
    address.sun_len = sizeof(address);
    memcpy(address.sun_path, argv[2], length + 1);
    int64_t began = now_ms();
    if (began < 0) goto done;
    int rc = connect(control, (const struct sockaddr *)&address, sizeof(address));
    if (rc < 0) {
        if (errno != EINPROGRESS || ready(control, POLLOUT, began + 2000) < 0) goto done;
        int error = 0;
        socklen_t size = sizeof(error);
        if (getsockopt(control, SOL_SOCKET, SO_ERROR, &error, &size) < 0 || size != sizeof(error) || error)
            goto done;
    }
    memcpy(packet, "FBSTTY01", 8);
    packet[8] = (unsigned char)term_length;
    memcpy(packet + 9, term, term_length);
    int64_t send_started = now_ms();
    if (send_started < 0) goto done;
    ssize_t sent;
    for (;;) {
        sent = faction_send_fd(control, tty, packet, sizeof(packet));
        if (sent >= 0) break;
        if (errno != EINTR && errno != EAGAIN && errno != EWOULDBLOCK) goto done;
        if (ready(control, POLLOUT, send_started + 2000) < 0) goto done;
    }
    if (sent <= 0 || (size_t)sent > sizeof(packet)) goto done;
    size_t used = (size_t)sent;
    while (used < sizeof(packet)) {
        if (ready(control, POLLOUT, send_started + 2000) < 0) goto done;
        ssize_t n = write(control, packet + used, sizeof(packet) - used);
        if (n > 0) used += (size_t)n;
        else if (n == 0 || (errno != EINTR && errno != EAGAIN && errno != EWOULDBLOCK)) goto done;
    }
    int64_t wait_started = now_ms();
    if (wait_started < 0) goto done;
    for (;;) {
        if (ready(control, POLLIN, wait_started + 1900000) < 0) goto done;
        unsigned char status;
        ssize_t n = read(control, &status, 1);
        if (n == 1) { if (status == 0) result = 0; break; }
        if (n == 0 || (errno != EINTR && errno != EAGAIN && errno != EWOULDBLOCK)) break;
    }
done:
    if (saved) (void)tcsetattr(tty, TCSAFLUSH, &original);
    if (control >= 0) close(control);
    if (tty >= 0) close(tty);
    return result;
}

int main(int argc, char **argv) {
    /* Rust std ignores SIGPIPE too; preserve restoration after broken control. */
    struct sigaction action;
    memset(&action, 0, sizeof(action));
    action.sa_handler = SIG_IGN;
    sigemptyset(&action.sa_mask);
    if (sigaction(SIGPIPE, &action, NULL) == 0 && run(argc, argv) == 0) return 0;
    static const char error[] = "Faction SSH: terminal admission denied or session ended.\n";
    (void)write(STDERR_FILENO, error, sizeof(error) - 1);
    return 1;
}
