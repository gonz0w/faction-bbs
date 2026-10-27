/* Test-only wrappers simulate refusal without changing host/library policy. */
#include <openssl/crypto.h>
#include <openssl/evp.h>
#include <sys/resource.h>
#include <assert.h>
#include <stdint.h>
#include <string.h>

static int refusal;
static unsigned version_calls, limit_calls, allocation_calls;
static unsigned int selected_major(void) {
    ++version_calls;
    return refusal == 4 ? 4 : OPENSSL_version_major();
}
static int selected_limit(int resource, const struct rlimit *limit) {
    ++limit_calls;
    return refusal == 1 ? -1 : setrlimit(resource, limit);
}
static EVP_CIPHER_CTX *selected_allocation(void) {
    ++allocation_calls;
    return EVP_CIPHER_CTX_new();
}
#define OPENSSL_version_major selected_major
#define setrlimit selected_limit
#define EVP_CIPHER_CTX_new selected_allocation
#include "../native/attachment.c"

int main(int argc, char **argv) {
    uint8_t key[32] = {0}, iv[12] = {0}, input[1] = {0}, output[33];
    size_t written = 99;
    assert(argc == 2);
    refusal = !strcmp(argv[1], "runtime4") ? 4 :
              !strcmp(argv[1], "core-refused") ? 1 : 0;
    memset(output, 0xa5, sizeof output);
    if (refusal) {
        assert(faction_blob_prepare() != 0);
        assert(faction_blob_crypt(1, key, iv, input, 1, output, sizeof output, &written) != 0);
        assert(written == 0 && allocation_calls == 0);
        for (size_t i = 0; i < sizeof output; ++i) assert(output[i] == 0);
        assert(version_calls >= 2);
        assert(refusal == 4 ? limit_calls == 0 : limit_calls >= 2);
    } else if (!strcmp(argv[1], "positive")) {
        struct rlimit core;
        assert(faction_blob_prepare() == 0);
        assert(getrlimit(RLIMIT_CORE, &core) == 0 && core.rlim_cur == 0 && core.rlim_max == 0);
        assert(faction_blob_crypt(1, key, iv, input, 1, output, sizeof output, &written) == 0);
        assert(written == 17 && allocation_calls == 1 && version_calls >= 2 && limit_calls >= 2);
    } else {
        /* Invalid capacities/modes/nulls refuse before context allocation. */
        assert(faction_blob_crypt(1, key, iv, input, 1, output, 16, &written) != 0);
        assert(written == 0 && allocation_calls == 0);
        for (size_t i = 0; i < 16; ++i) assert(output[i] == 0);
        for (size_t i = 16; i < sizeof output; ++i) assert(output[i] == 0xa5);
        assert(faction_blob_crypt(1, key, iv, input, BLOB_MAX + 1u, output, sizeof output, &written) != 0);
        assert(faction_blob_crypt(0, key, iv, input, 1, output, sizeof output, &written) != 0);
        assert(faction_blob_crypt(2, key, iv, input, 1, output, sizeof output, &written) != 0);
        assert(faction_blob_crypt(1, NULL, iv, input, 1, output, sizeof output, &written) != 0);
        assert(faction_blob_crypt(1, key, NULL, input, 1, output, sizeof output, &written) != 0);
        assert(faction_blob_crypt(1, key, iv, NULL, 1, output, sizeof output, &written) != 0);
        assert(faction_blob_crypt(1, key, iv, input, 1, NULL, sizeof output, &written) != 0);
        assert(faction_blob_crypt(1, key, iv, input, 1, output, sizeof output, NULL) != 0);
        assert(allocation_calls == 0);
    }
    return 0;
}
