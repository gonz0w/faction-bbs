/* Faction BBS, BSD-3-Clause.
 * RFC4252 section 7 request construction, matching OpenSSH 10.3p1
 * auth2-pubkey.c's ordinary publickey path. Synthetic session identifiers;
 * this component oracle does not execute KEX or sshd's authorization policy.
 */
#ifndef FACTION_REPLAY_REQUEST_H
#define FACTION_REPLAY_REQUEST_H
#include "ssh2.h"
static struct sshbuf *replay_request(const unsigned char *sid, size_t sid_len,
    const char *user, const char *service, const struct sshkey *key) {
    struct sshbuf *b = sshbuf_new(), *public = sshbuf_new();
    if (b == NULL || public == NULL ||
        sshkey_putb(key, public) != 0 ||
        sshbuf_put_string(b, sid, sid_len) != 0 ||
        sshbuf_put_u8(b, SSH2_MSG_USERAUTH_REQUEST) != 0 ||
        sshbuf_put_cstring(b, user) != 0 ||
        sshbuf_put_cstring(b, service) != 0 ||
        sshbuf_put_cstring(b, "publickey") != 0 ||
        sshbuf_put_u8(b, 1) != 0 ||
        sshbuf_put_cstring(b, "ssh-ed25519") != 0 ||
        sshbuf_put_stringb(b, public) != 0) {
        fputs("request construction failed\n", stderr);
        exit(2);
    }
    sshbuf_free(public);
    return b;
}
#endif
