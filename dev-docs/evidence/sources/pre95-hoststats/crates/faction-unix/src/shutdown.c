/* Process-wide cooperative shutdown notification. No allocation or I/O in handler. */
#define _POSIX_C_SOURCE 200809L
#include <signal.h>
#include <stdatomic.h>
#include <string.h>
_Static_assert(ATOMIC_INT_LOCK_FREE == 2, "signal flag must be lock-free");
static _Atomic int stopping;
static void stop_handler(int signo) {
    (void)signo;
    atomic_store_explicit(&stopping, 1, memory_order_relaxed);
}
int faction_install_shutdown(void) {
    struct sigaction action;
    memset(&action, 0, sizeof(action));
    action.sa_handler = stop_handler;
    sigemptyset(&action.sa_mask);
    if (sigaction(SIGTERM, &action, NULL) == -1 ||
        sigaction(SIGINT, &action, NULL) == -1 ||
        sigaction(SIGHUP, &action, NULL) == -1) return -1;
    return 0;
}
int faction_shutdown_requested(void) {
    return atomic_load_explicit(&stopping, memory_order_relaxed);
}
