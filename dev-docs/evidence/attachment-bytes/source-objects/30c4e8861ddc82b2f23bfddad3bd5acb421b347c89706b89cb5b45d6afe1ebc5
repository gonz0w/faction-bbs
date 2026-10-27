/* Isolated issue #250 byte proof. Public laboratory fixtures only.
 * stdin: 32-byte key, 12-byte IV, then bounded payload to EOF; empty AAD.
 * stdout: ciphertext || 16-byte tag, or plaintext after authentication only.
 */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <openssl/crypto.h>
#include <openssl/evp.h>

#define MAX_PLAINTEXT 8192u
#define TAG_BYTES 16u
#define MAX_CIPHERTEXT (MAX_PLAINTEXT + TAG_BYTES)
#define HEADER_BYTES 44u

int main(int argc, char **argv)
{
    unsigned char frame[HEADER_BYTES + MAX_CIPHERTEXT + 1u];
    unsigned char output[MAX_CIPHERTEXT + EVP_MAX_BLOCK_LENGTH];
    EVP_CIPHER *cipher = NULL;
    EVP_CIPHER_CTX *context = NULL;
    size_t count, payload, data_length, offset, written = 0, chunk = MAX_PLAINTEXT;
    int encrypt, amount = 0, status = 4;
    const char *diagnostic = "crypto operation failed\n";

    if (argc < 2 || argc > 3 ||
        (strcmp(argv[1], "encrypt") != 0 && strcmp(argv[1], "decrypt") != 0)) {
        status = 2;
        diagnostic = "invalid request\n";
        goto done;
    }
    encrypt = strcmp(argv[1], "encrypt") == 0;
    if (argc == 3) {
        char *end;
        unsigned long parsed;
        const unsigned char *digit = (const unsigned char *)argv[2];
        if (*digit == '\0') {
            status = 2;
            diagnostic = "invalid request\n";
            goto done;
        }
        for (; *digit != '\0'; ++digit) {
            if (*digit < '0' || *digit > '9') {
                status = 2;
                diagnostic = "invalid request\n";
                goto done;
            }
        }
        errno = 0;
        parsed = strtoul(argv[2], &end, 10);
        if (errno != 0 || *end != '\0' || parsed < 1 || parsed > MAX_PLAINTEXT) {
            status = 2;
            diagnostic = "invalid request\n";
            goto done;
        }
        chunk = (size_t)parsed;
    }
    count = fread(frame, 1, sizeof(frame), stdin);
    if (ferror(stdin) || count < HEADER_BYTES || count == sizeof(frame)) {
        status = 2;
        diagnostic = "invalid request\n";
        goto done;
    }
    payload = count - HEADER_BYTES;
    if ((encrypt && payload > MAX_PLAINTEXT) ||
        (!encrypt && (payload < TAG_BYTES || payload > MAX_CIPHERTEXT))) {
        status = 2;
        diagnostic = "invalid request\n";
        goto done;
    }
    data_length = encrypt ? payload : payload - TAG_BYTES;
    cipher = EVP_CIPHER_fetch(NULL, "AES-256-GCM", "provider=default");
    context = EVP_CIPHER_CTX_new();
    if (cipher == NULL || context == NULL ||
        EVP_CipherInit_ex2(context, cipher, NULL, NULL, encrypt, NULL) != 1 ||
        EVP_CIPHER_CTX_ctrl(context, EVP_CTRL_GCM_SET_IVLEN, 12, NULL) != 1 ||
        EVP_CipherInit_ex2(context, NULL, frame, frame + 32, encrypt, NULL) != 1)
        goto done;
    for (offset = 0; offset < data_length;) {
        size_t selected = data_length - offset;
        int updated;
        if (selected > chunk)
            selected = chunk;
        /* Both selected and the total are <= 8192 before conversion to int. */
        updated = encrypt
            ? EVP_EncryptUpdate(context, output + written, &amount,
                                frame + HEADER_BYTES + offset, (int)selected)
            : EVP_DecryptUpdate(context, output + written, &amount,
                                frame + HEADER_BYTES + offset, (int)selected);
        if (updated != 1 || amount < 0 || (size_t)amount > sizeof(output) - written)
            goto done;
        written += (size_t)amount;
        offset += selected;
    }
    if (encrypt) {
        if (EVP_EncryptFinal_ex(context, output + written, &amount) != 1)
            goto done;
    } else {
        if (EVP_CIPHER_CTX_ctrl(context, EVP_CTRL_GCM_SET_TAG, TAG_BYTES,
                               frame + HEADER_BYTES + data_length) != 1)
            goto done;
        if (EVP_DecryptFinal_ex(context, output + written, &amount) <= 0) {
            status = 3;
            diagnostic = "authentication failed\n";
            goto done;
        }
    }
    if (amount < 0 || (size_t)amount > sizeof(output) - written)
        goto done;
    written += (size_t)amount;
    if (written != data_length)
        goto done;
    if (encrypt) {
        if (EVP_CIPHER_CTX_ctrl(context, EVP_CTRL_GCM_GET_TAG, TAG_BYTES,
                               output + written) != 1)
            goto done;
        written += TAG_BYTES;
    }
    /* This is the only plaintext release, after positive decrypt finalization. */
    if (fwrite(output, 1, written, stdout) != written || fflush(stdout) != 0) {
        diagnostic = "output failed\n";
        goto done;
    }
    status = 0;
done:
    EVP_CIPHER_CTX_free(context);
    EVP_CIPHER_free(cipher);
    OPENSSL_cleanse(frame, sizeof(frame));
    OPENSSL_cleanse(output, sizeof(output));
    if (status != 0)
        fputs(diagnostic, stderr);
    return status;
}
