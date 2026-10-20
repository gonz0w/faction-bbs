/* Synthetic allocation ownership probe, not a server or production language choice.
 * Fixed workload: 250 sessions, 256 records of 256 bytes per session.
 * Both strategies deliver identical bytes to each session; all allocated bytes touched.
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { SESSIONS = 250, RECORDS = 256, RECORD_BYTES = 256 };
static void *checked_malloc(size_t bytes) {
    void *result = malloc(bytes);
    if (!result) { fputs("allocation failed\n", stderr); exit(1); }
    return result;
}
int main(int argc, char **argv) {
    if (argc != 2 || (strcmp(argv[1], "copy") && strcmp(argv[1], "shared"))) {
        fputs("usage: probe copy|shared\n", stderr); return 2;
    }
    const int shared = !strcmp(argv[1], "shared");
    const size_t payload = (size_t)RECORDS * RECORD_BYTES;
    size_t allocated = 0;
    unsigned char **copies = NULL, *pool = NULL;
    const unsigned char **refs = NULL;
    if (shared) {
        pool = checked_malloc(payload); allocated += payload;
        refs = checked_malloc((size_t)SESSIONS * RECORDS * sizeof(*refs));
        allocated += (size_t)SESSIONS * RECORDS * sizeof(*refs);
        for (size_t i = 0; i < payload; ++i) pool[i] = (unsigned char)(i % 251);
        for (size_t i = 0; i < (size_t)SESSIONS * RECORDS; ++i)
            refs[i] = pool + (i % RECORDS) * RECORD_BYTES;
    } else {
        copies = checked_malloc(SESSIONS * sizeof(*copies)); allocated += SESSIONS * sizeof(*copies);
        for (size_t s = 0; s < SESSIONS; ++s) {
            copies[s] = checked_malloc(payload); allocated += payload;
            for (size_t i = 0; i < payload; ++i) copies[s][i] = (unsigned char)(i % 251);
        }
    }
    /* Volatile prevents optimizing away the walk; checksum checks equal logical delivery. */
    volatile uint64_t checksum = 0;
    for (size_t s = 0; s < SESSIONS; ++s)
        for (size_t r = 0; r < RECORDS; ++r)
            for (size_t b = 0; b < RECORD_BYTES; ++b)
                checksum += shared ? refs[s * RECORDS + r][b] : copies[s][r * RECORD_BYTES + b];
    printf("{\"mode\":\"%s\",\"sessions\":%d,\"pointer_bytes\":%zu,"
           "\"requested_live_bytes\":%zu,\"checksum\":%llu}\n",
           argv[1], SESSIONS, sizeof(void *), allocated, (unsigned long long)checksum);
    if (shared) { free(refs); free(pool); }
    else { for (size_t s = 0; s < SESSIONS; ++s) free(copies[s]); free(copies); }
    return 0;
}
