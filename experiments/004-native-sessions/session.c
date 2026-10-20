/* Local research only; ALLOW is a synthetic harness control, not authentication. */
#include <sys/socket.h>
#include <sys/un.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
struct session { int fd, approved; size_t in_n, out_n; char in[1025], out[8192]; };
static struct session sessions[250];
static void drop(struct session *s) { close(s->fd); memset(s, 0, sizeof(*s)); s->fd = -1; }
int main(int argc, char **argv) {
    if (argc != 2 || strlen(argv[1]) >= sizeof(((struct sockaddr_un *)0)->sun_path)) return 2;
    signal(SIGPIPE, SIG_IGN);
    int listener = socket(AF_UNIX, SOCK_STREAM, 0);
    if (listener < 0) return 1;
    struct sockaddr_un addr; memset(&addr, 0, sizeof(addr)); addr.sun_family = AF_UNIX;
    strcpy(addr.sun_path, argv[1]);
    if (bind(listener, (struct sockaddr *)&addr, sizeof(addr)) || listen(listener, 256)) return 1;
    if (fcntl(listener, F_SETFL, O_NONBLOCK)) return 1;
    for (size_t i = 0; i < 250; ++i) sessions[i].fd = -1;
    for (;;) {
        struct pollfd waitfd = { listener, POLLIN, 0 };
        (void)poll(&waitfd, 1, 1); /* Same 1ms scan style as Rust; deliberately simple. */
        int fd = accept(listener, NULL, NULL);
        if (fd >= 0) {
            size_t i = 0; while (i < 250 && sessions[i].fd >= 0) ++i;
            if (i == 250) { (void)write(fd, "FULL\n", 5); close(fd); }
            else if (fcntl(fd, F_SETFL, O_NONBLOCK)) close(fd);
            else sessions[i].fd = fd;
        }
        for (size_t i = 0; i < 250; ++i) {
            struct session *s = &sessions[i]; if (s->fd < 0) continue;
            char block[256]; ssize_t n = read(s->fd, block, sizeof(block));
            if (n == 0 || (n < 0 && errno != EAGAIN && errno != EWOULDBLOCK)) { drop(s); continue; }
            for (ssize_t j = 0; j < n && s->fd >= 0; ++j) {
                unsigned char b = (unsigned char)block[j];
                if (b != '\n') {
                    if (b < 32 || b > 126 || s->in_n >= 1024) { drop(s); break; }
                    s->in[s->in_n++] = (char)b; continue;
                }
                s->in[s->in_n] = 0;
                const char *response = "ERROR\n";
                if (!strcmp(s->in, "ALLOW")) { s->approved = 1; response = "OK\n"; }
                else if (!strcmp(s->in, "REVOKE")) { s->approved = 0; response = "OK\n"; }
                else if (!strcmp(s->in, "READ") || !strcmp(s->in, "POST") || !strcmp(s->in, "CHAT")) {
                    if (!s->approved) response = "DENIED\n";
                    else if (!strcmp(s->in, "READ")) response = "ROOM synthetic local message\n";
                    else if (!strcmp(s->in, "POST")) response = "POSTED synthetic only\n";
                    else response = "CHAT synthetic only\n";
                }
                size_t length = strlen(response);
                if (s->out_n + length > sizeof(s->out)) { drop(s); break; }
                memcpy(s->out + s->out_n, response, length); s->out_n += length; s->in_n = 0;
            }
            if (s->fd >= 0 && s->out_n) {
                ssize_t sent = write(s->fd, s->out, s->out_n);
                if (sent > 0) { memmove(s->out, s->out + sent, s->out_n - (size_t)sent); s->out_n -= (size_t)sent; }
                else if (sent == 0 || (errno != EAGAIN && errno != EWOULDBLOCK)) drop(s);
            }
        }
    }
}
