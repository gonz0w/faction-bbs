/* Faction BBS, BSD-3-Clause. SSH signature component oracle, not a wire replay. */
#include "includes.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "sshbuf.h"
#include "sshkey.h"
#include "ssherr.h"
#include "replay-request.h"

static void must(int value, const char *where) {
    if (value != 0) {
        fprintf(stderr, "%s: %s\n", where, ssh_err(value));
        exit(2);
    }
}
static void check(const struct sshkey *key, const unsigned char *sig, size_t len,
    struct sshbuf *request, int accept, const char *name) {
    int r = sshkey_verify(key, sig, len, sshbuf_ptr(request),
        sshbuf_len(request), "ssh-ed25519", 0, NULL);
    if ((r == 0) != accept || (!accept && r != SSH_ERR_SIGNATURE_INVALID)) {
        fprintf(stderr, "unexpected verification: %s code=%d\n", name, r);
        exit(3);
    }
    sshbuf_free(request);
}
int main(void) {
    struct sshkey *key = NULL, *other = NULL, *public = NULL;
    struct sshbuf *request;
    unsigned char sid[32], changed[32], *sig = NULL;
    size_t len = 0;
    memset(sid, 0x23, sizeof(sid));
    memcpy(changed, sid, sizeof(sid)); changed[0] ^= 1;
    must(sshkey_generate(KEY_ED25519, 0, &key), "generate");
    must(sshkey_generate(KEY_ED25519, 0, &other), "other key");
    must(sshkey_from_private(key, &public), "public key");
    request = replay_request(sid, sizeof(sid), "r223-a", "ssh-connection", key);
    must(sshkey_sign(key, &sig, &len, sshbuf_ptr(request), sshbuf_len(request),
        "ssh-ed25519", NULL, NULL, 0), "sign");
    check(public, sig, len, request, 1, "original");
    check(public, sig, len, replay_request(sid, sizeof(sid), "r223-a",
        "ssh-connection", key), 1, "same request reconstruction");
    check(public, sig, len, replay_request(changed, sizeof(changed), "r223-a",
        "ssh-connection", key), 0, "different SSH session identifier");
    check(public, sig, len, replay_request(sid, sizeof(sid), "r223-b",
        "ssh-connection", key), 0, "different username");
    check(public, sig, len, replay_request(sid, sizeof(sid), "r223-a",
        "other-service", key), 0, "different service");
    check(public, sig, len, replay_request(sid, sizeof(sid), "r223-a",
        "ssh-connection", other), 0, "different request key");
    check(other, sig, len, replay_request(sid, sizeof(sid), "r223-a",
        "ssh-connection", key), 0, "different verifying key");
    free(sig); sshkey_free(public); sshkey_free(key); sshkey_free(other);
    puts("{\"result\":\"pass\",\"layer\":\"OpenSSH RFC4252 public-key signature component\",\"checks\":7,\"wire_replay_claim\":false}");
    return 0;
}
