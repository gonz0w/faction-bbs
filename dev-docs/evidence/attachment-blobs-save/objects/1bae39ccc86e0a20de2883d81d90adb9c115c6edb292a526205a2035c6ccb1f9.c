/* Bounded member-device blob EVP glue, BSD-3-Clause. No algorithms here.
 * Private Rust FFI owns valid, nonoverlapping pointers and declared capacities.
 * Output remains quarantined by Rust until authentication/size/hash validation.
 */
#include <limits.h>
#include <stdint.h>
#include <sys/resource.h>
#include <openssl/crypto.h>
#include <openssl/evp.h>
#include <openssl/opensslv.h>
#if OPENSSL_VERSION_MAJOR != 3
#error "faction-omemo blobs require native system OpenSSL major 3"
#endif

#define BLOB_MAX 1048576u
#define TAG_BYTES 16u
#define CIPHER_MAX (BLOB_MAX + TAG_BYTES)

/* Blob-only callers must establish this before allocating new private buffers. */
int faction_blob_prepare(void) {
    const struct rlimit core = {0, 0};
    struct rlimit observed;
    return OPENSSL_version_major() == 3 &&
           setrlimit(RLIMIT_CORE, &core) == 0 &&
           getrlimit(RLIMIT_CORE, &observed) == 0 &&
           observed.rlim_cur == 0 && observed.rlim_max == 0 ? 0 : -1;
}

/* Both operations require data-length +16 capacity, including final scratch.
 * Successful encryption appends tag16; successful decryption returns no tag.
 * All failures clear the declared bounded output and reset its written length.
 */
int faction_blob_crypt(int encrypt, const uint8_t *key, const uint8_t *iv,
                       const uint8_t *input, size_t n, uint8_t *output,
                       size_t capacity, size_t *written) {
    EVP_CIPHER *cipher = NULL;
    EVP_CIPHER_CTX *context = NULL;
    size_t data_length;
    int amount = 0, final_amount = 0, result = -1;
    if (written) *written = 0;
    if (!output || capacity > CIPHER_MAX) return -1;
    if (!written || !key || !iv || (!input && n) ||
        (encrypt != 0 && encrypt != 1) ||
        (encrypt ? n > BLOB_MAX : n < TAG_BYTES || n > CIPHER_MAX))
        goto done;
    data_length = encrypt ? n : n - TAG_BYTES;
    if (data_length > INT_MAX || capacity < data_length + TAG_BYTES ||
        faction_blob_prepare() != 0)
        goto done;
    cipher = EVP_CIPHER_fetch(NULL, "AES-256-GCM", "provider=default");
    context = EVP_CIPHER_CTX_new();
    if (!cipher || !context ||
        EVP_CipherInit_ex2(context, cipher, NULL, NULL, encrypt, NULL) != 1 ||
        EVP_CIPHER_CTX_ctrl(context, EVP_CTRL_GCM_SET_IVLEN, 12, NULL) != 1 ||
        EVP_CipherInit_ex2(context, NULL, key, iv, encrypt, NULL) != 1)
        goto done;
    /* Empty AAD. Complete input is bounded before size_t -> int conversion. */
    if (data_length &&
        EVP_CipherUpdate(context, output, &amount, input, (int)data_length) != 1)
        goto done;
    if (amount < 0 || (size_t)amount != data_length) goto done;
    if (!encrypt &&
        EVP_CIPHER_CTX_ctrl(context, EVP_CTRL_GCM_SET_TAG, TAG_BYTES,
                            (void *)(input + data_length)) != 1)
        goto done;
    if (EVP_CipherFinal_ex(context, output + data_length, &final_amount) != 1 ||
        final_amount != 0)
        goto done;
    if (encrypt &&
        EVP_CIPHER_CTX_ctrl(context, EVP_CTRL_GCM_GET_TAG, TAG_BYTES,
                            output + data_length) != 1)
        goto done;
    *written = data_length + (encrypt ? TAG_BYTES : 0);
    result = 0;
done:
    EVP_CIPHER_CTX_free(context);
    EVP_CIPHER_free(cipher);
    if (result) OPENSSL_cleanse(output, capacity);
    return result;
}

int faction_blob_sha256(const uint8_t *input, size_t n, uint8_t *hash) {
    unsigned int written = 0;
    if (!hash) return -1;
    if ((!input && n) || n > BLOB_MAX || faction_blob_prepare() != 0 ||
        EVP_Digest(input, n, hash, &written, EVP_sha256(), NULL) != 1 || written != 32) {
        OPENSSL_cleanse(hash, 32);
        return -1;
    }
    return 0;
}
