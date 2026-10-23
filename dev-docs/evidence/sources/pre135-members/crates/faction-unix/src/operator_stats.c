/* Read-only local host counters. No pointer escapes, allocation, helper or writes. */
#define _NETBSD_SOURCE 1
#define _DARWIN_C_SOURCE 1
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#if defined(__NetBSD__) || defined(__APPLE__)
#include <sys/types.h>
#include <sys/sysctl.h>
#include <sys/socket.h>
#include <net/if.h>
#include <net/route.h>
#include <stdlib.h>
#include <math.h>
#ifdef __NetBSD__
#include <uvm/uvm_extern.h>
#include <sys/sched.h>
#else
#include <mach/mach.h>
#include <mach/mach_host.h>
#endif

static int scalar(const char *name, void *data, size_t capacity) {
    size_t length = capacity;
    return sysctlbyname(name, data, &length, NULL, 0) == 0 && length == capacity;
}
/* Only IFINFO counters leave this function. Routing addresses are never exposed.
 * One fixed 64-KiB syscall buffer; larger/changing tables fail this metric only. */
static int network(uint64_t out[64]) {
    unsigned char bytes[65536];
    size_t size = sizeof(bytes), at = 0, count = 0;
#ifdef __APPLE__
    int mib[] = {CTL_NET, PF_ROUTE, 0, AF_UNSPEC, NET_RT_IFLIST2, 0};
#else
    int mib[] = {CTL_NET, PF_ROUTE, 0, AF_UNSPEC, NET_RT_IFLIST, 0};
#endif
    if (sysctl(mib, 6, bytes, &size, NULL, 0) != 0 || size > sizeof(bytes)) return 0;
    while (at < size) {
        uint16_t length;
        if (size - at < 4) return 0;
        memcpy(&length, bytes + at, sizeof(length));
        if (length < 4 || length > size - at) return 0;
#ifdef __APPLE__
        if (bytes[at+3] == RTM_IFINFO2) {
            struct if_msghdr2 h;
#else
        if (bytes[at+3] == RTM_IFINFO) {
            struct if_msghdr h;
#endif
            if (length < sizeof(h)) return 0;
            memcpy(&h, bytes + at, sizeof(h));
            if (h.ifm_version != RTM_VERSION) return 0;
            if ((h.ifm_flags & IFF_LOOPBACK) == 0) {
                if (count == 16 || h.ifm_index == 0) return 0;
                for (size_t i = 0; i < count; i++) if (out[16+i*3] == h.ifm_index) return 0;
                out[16+count*3] = h.ifm_index;
                out[17+count*3] = h.ifm_data.ifi_ibytes;
                out[18+count*3] = h.ifm_data.ifi_obytes;
                count++;
            }
        }
        at += length;
    }
    out[15] = count;
    return 1;
}
#endif

/* Slots: flags, CPUs, physical, free, 5 CPU ticks (idle last), 3 milli-loads;
 * interface count at 15 and at most 16 (index,RX,TX) triples from slot 16.
 * Flags: basic=1, memory=2, CPU=4, load=8, network=16. Failure is per metric. */
int faction_operator_stats(uint64_t *out, size_t capacity) {
    if (out == NULL || capacity != 64) return -1;
    memset(out, 0, 64 * sizeof(*out));
#if defined(__NetBSD__) || defined(__APPLE__)
    uint32_t cpus = 0;
    uint64_t physical = 0;
#ifdef __NetBSD__
    const char *memory = "hw.physmem64";
#else
    const char *memory = "hw.memsize";
#endif
    if (scalar("hw.ncpu", &cpus, sizeof(cpus)) && cpus > 0 &&
        scalar(memory, &physical, sizeof(physical)) && physical > 0) {
        out[0] |= 1; out[1] = cpus; out[2] = physical;
    }
#ifdef __NetBSD__
    struct uvmexp_sysctl vm;
    if (scalar("vm.uvmexp2", &vm, sizeof(vm)) && vm.pagesize > 0 && vm.free >= 0 &&
        (uint64_t)vm.free <= UINT64_MAX / (uint64_t)vm.pagesize) {
        uint64_t free_bytes = (uint64_t)vm.free * (uint64_t)vm.pagesize;
        if (physical > 0 && free_bytes <= physical) { out[0] |= 2; out[3] = free_bytes; }
    }
    uint64_t ticks[CPUSTATES];
    _Static_assert(CPUSTATES == 5 && CP_IDLE == 4, "review CPU state layout");
    if (scalar("kern.cp_time", ticks, sizeof(ticks))) {
        memcpy(out+4, ticks, sizeof(ticks)); out[0] |= 4;
    }
#else
    host_t host = mach_host_self();
    vm_statistics64_data_t vm;
    mach_msg_type_number_t vcount = HOST_VM_INFO64_COUNT;
    vm_size_t page = 0;
    if (host_page_size(host, &page) == KERN_SUCCESS &&
        host_statistics64(host, HOST_VM_INFO64, (host_info64_t)&vm, &vcount) == KERN_SUCCESS &&
        vcount == HOST_VM_INFO64_COUNT && page > 0 &&
        (uint64_t)vm.free_count <= UINT64_MAX / (uint64_t)page) {
        uint64_t free_bytes = (uint64_t)vm.free_count * (uint64_t)page;
        if (physical > 0 && free_bytes <= physical) { out[0] |= 2; out[3] = free_bytes; }
    }
    host_cpu_load_info_data_t cpu;
    mach_msg_type_number_t ccount = HOST_CPU_LOAD_INFO_COUNT;
    if (host_statistics(host, HOST_CPU_LOAD_INFO, (host_info_t)&cpu, &ccount) == KERN_SUCCESS && ccount == HOST_CPU_LOAD_INFO_COUNT) {
        out[4] = cpu.cpu_ticks[CPU_STATE_USER]; out[5] = cpu.cpu_ticks[CPU_STATE_NICE];
        out[6] = cpu.cpu_ticks[CPU_STATE_SYSTEM]; out[8] = cpu.cpu_ticks[CPU_STATE_IDLE];
        out[0] |= 4;
    }
    (void)mach_port_deallocate(mach_task_self(), host);
#endif
    double load[3];
    if (getloadavg(load, 3) == 3) {
        int valid = 1;
        for (size_t i = 0; i < 3; i++) {
            if (!isfinite(load[i]) || load[i] < 0 || load[i] > 1000000) { valid = 0; break; }
            out[9+i] = (uint64_t)(load[i] * 1000);
        }
        if (valid) out[0] |= 8;
    }
    if (network(out)) out[0] |= 16;
#endif
    return 0;
}
