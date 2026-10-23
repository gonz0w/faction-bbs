/* BSD-3-Clause. Use the native NetBSD CMSG ABI, not Rust libc's aarch64 macros.
 * Fixed bounded buffers; no allocation, command execution or retained pointers.
 * These functions borrow socket/input fds; only newly received rights are closed.
 */
#include <sys/types.h>
#include <sys/socket.h>
#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <unistd.h>

ssize_t faction_send_fd(int socket_fd, int terminal, const unsigned char *bytes,
                       size_t length) {
    union { struct cmsghdr alignment; unsigned char bytes[256]; } control;
    struct msghdr message;
    struct iovec vector;
    struct cmsghdr *header;
    memset(&message, 0, sizeof(message));
    memset(&control, 0, sizeof(control));
    vector.iov_base = (void *)bytes;
    vector.iov_len = length;
    message.msg_iov = &vector;
    message.msg_iovlen = 1;
    message.msg_control = control.bytes;
    message.msg_controllen = CMSG_SPACE(sizeof(int));
    if (message.msg_controllen > sizeof(control.bytes)) { errno = EINVAL; return -1; }
    header = CMSG_FIRSTHDR(&message);
    header->cmsg_level = SOL_SOCKET;
    header->cmsg_type = SCM_RIGHTS;
    header->cmsg_len = CMSG_LEN(sizeof(int));
    memcpy(CMSG_DATA(header), &terminal, sizeof(terminal));
    return sendmsg(socket_fd, &message, 0);
}

ssize_t faction_recv_fd(int socket_fd, unsigned char *bytes, size_t length,
                       int *terminal) {
    union { struct cmsghdr alignment; unsigned char bytes[256]; } control;
    struct msghdr message;
    struct iovec vector;
    struct cmsghdr *header;
    int descriptors[8], count = 0, invalid = 0;
    ssize_t result;
    *terminal = -1;
    memset(&message, 0, sizeof(message));
    memset(&control, 0, sizeof(control));
    vector.iov_base = bytes;
    vector.iov_len = length;
    message.msg_iov = &vector;
    message.msg_iovlen = 1;
    message.msg_control = control.bytes;
    message.msg_controllen = sizeof(control.bytes);
    result = recvmsg(socket_fd, &message, MSG_CMSG_CLOEXEC);
    if (result < 0) return -1;
    if ((message.msg_flags & (MSG_CTRUNC | MSG_TRUNC)) != 0) invalid = 1;
    for (header = CMSG_FIRSTHDR(&message); header != NULL;
         header = CMSG_NXTHDR(&message, header)) {
        size_t size, index;
        if (header->cmsg_len < CMSG_LEN(0)) { invalid = 1; break; }
        size = header->cmsg_len - CMSG_LEN(0);
        if (header->cmsg_level != SOL_SOCKET || header->cmsg_type != SCM_RIGHTS ||
            size % sizeof(int) != 0) { invalid = 1; continue; }
        for (index = 0; index < size / sizeof(int); index++) {
            int fd;
            memcpy(&fd, CMSG_DATA(header) + index * sizeof(int), sizeof(fd));
            if (count < 8) descriptors[count++] = fd;
            else { close(fd); invalid = 1; }
        }
    }
    if (invalid || count != 1 || result == 0) {
        int index;
        for (index = 0; index < count; index++) close(descriptors[index]);
        errno = EPROTO;
        return -1;
    }
    if (fcntl(descriptors[0], F_SETFD, FD_CLOEXEC) < 0) {
        int saved = errno;
        close(descriptors[0]); errno = saved; return -1;
    }
    *terminal = descriptors[0];
    return result;
}
