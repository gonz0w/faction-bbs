/* Bounded research adapter, BSD-3-Clause. No cryptographic algorithms here.
 * The Rust caller serializes every entry: Picomemo callbacks are process-global.
 * All pointers/buffer capacities are owned and checked by the private Rust FFI.
 */
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/resource.h>
#include <unistd.h>
#include <string.h>
#include "omemo2.h"
#include "driver.h"

#define PLAIN_MAX 8192u
#define PAYLOAD_MAX (PLAIN_MAX + 16u)
#define KEY_MAX 256u
#define SNAP_MAX 32768u
#define SKIP_MAX 32u
#define ERROR (-1)

struct proof {
    struct omemo2Store store;
    struct omemo2Session session;
    uint8_t pin[32];
    unsigned pinned;
    unsigned count;
    struct omemo2MessageKey skipped[SKIP_MAX];
};
static struct proof *active;

static void wipe(void *p, size_t n) { omemoDriverWipe(p, n); }
static int same(const void *a, const void *b, size_t n) {
    return !omemoDriverCompare(a, b, n);
}
static int load_skipped(struct omemo2Session *session, struct omemo2MessageKey *key) {
    (void)session;
    if (!active) return ERROR;
    for (unsigned i = 0; i < active->count; ++i) {
        if (active->skipped[i].nr == key->nr && same(active->skipped[i].dh,key->dh,32)) {
            memcpy(key->mk, active->skipped[i].mk,32);
            --active->count;
            memmove(active->skipped+i,active->skipped+i+1,
                    (active->count-i)*sizeof(*key));
            wipe(active->skipped+active->count,sizeof(*key));
            return 0;
        }
    }
    return 1;
}
static int store_skipped(struct omemo2Session *session,
                         const struct omemo2MessageKey *key, uint64_t remaining) {
    (void)session;
    if (!active || remaining > SKIP_MAX || active->count >= SKIP_MAX) return ERROR;
    for (unsigned i=0; i<active->count; ++i)
        if (active->skipped[i].nr == key->nr && same(active->skipped[i].dh,key->dh,32))
            return ERROR;
    memcpy(active->skipped+active->count++,key,sizeof(*key));
    return 0;
}
static void enter(struct proof *ctx) {
    active=ctx;
    omemo2SetCallbacks(load_skipped,store_skipped,NULL);
}
static int discard_probe_key(struct omemo2Session *session,
                             const struct omemo2MessageKey *key,uint64_t remaining) {
    (void)session;(void)key;
    // Authentication-only rederivation: no stored keys, at most64 derivations.
    return remaining>64 ? ERROR : 0;
}
int proof_private_pipes(void) {
    struct stat input,output;
    struct rlimit core={0,0};
    if (fstat(STDIN_FILENO,&input) || fstat(STDOUT_FILENO,&output)) return ERROR;
    if (!(S_ISFIFO(input.st_mode) || S_ISSOCK(input.st_mode)) ||
        !(S_ISFIFO(output.st_mode) || S_ISSOCK(output.st_mode))) return ERROR;
    return setrlimit(RLIMIT_CORE,&core) ? ERROR : 0;
}
static void leave(void) {
    active=NULL;
    omemo2SetCallbacks(NULL,NULL,NULL);
}
void *proof_alloc(void) { return calloc(1,sizeof(struct proof)); }
void proof_free(void *ptr) {
    if (ptr) { wipe(ptr,sizeof(struct proof)); free(ptr); }
}
void *proof_clone(const void *ptr) {
    struct proof *copy = proof_alloc();
    if (copy && ptr) memcpy(copy,ptr,sizeof(*copy));
    return copy;
}
int proof_setup(void *ptr) {
    struct proof *ctx=ptr;
    if (!ctx || ctx->store.init) return ERROR;
    struct proof tmp={0};
    int result=omemo2SetupStore(&tmp.store);
    if (!result) memcpy(ctx,&tmp,sizeof(tmp));
    wipe(&tmp,sizeof(tmp));
    return result;
}
int proof_bundle(const void *ptr, uint8_t out[160], uint32_t ids[2]) {
    const struct proof *ctx=ptr;
    memset(out,0,160); memset(ids,0,2*sizeof(*ids));
    if (!ctx || !ctx->store.init) return ERROR;
    const struct omemo2PreKey *pk=NULL;
    for (unsigned i=0;i<OMEMO2_NUMPREKEYS;++i)
        if (ctx->store.prekeys[i].id) {pk=ctx->store.prekeys+i;break;}
    if (!pk) return ERROR;
    omemo2SerializeKey(out,ctx->store.identity.pub);
    omemo2SerializeKey(out+32,ctx->store.cursignedprekey.kp.pub);
    memcpy(out+64,ctx->store.cursignedprekey.sig,64);
    omemo2SerializeKey(out+128,pk->kp.pub);
    ids[0]=ctx->store.cursignedprekey.id; ids[1]=pk->id;
    return 0;
}
int proof_trust(void *ptr, const uint8_t pin[32]) {
    struct proof *ctx=ptr;
    if (!ctx || !ctx->store.init || !pin) return ERROR;
    if (ctx->pinned && !same(ctx->pin,pin,32)) return ERROR;
    memcpy(ctx->pin,pin,32); ctx->pinned=1;
    return 0;
}
int proof_start(void *ptr, const uint8_t bundle[160], const uint32_t ids[2]) {
    struct proof *ctx=ptr;
    if (!ctx || !ctx->pinned || !ctx->store.init || ctx->session.init ||
        !ids[0] || ids[0]>0x7fffffff || !ids[1] || ids[1]>0x7fffffff ||
        !same(ctx->pin,bundle,32)) return ERROR;
    struct proof tmp=*ctx;
    int result=omemo2InitiateSession(&tmp.session,&tmp.store,bundle+64,
                                   bundle+32,bundle,bundle+128,ids[0],ids[1]);
    if (!result) memcpy(ctx,&tmp,sizeof(tmp));
    wipe(&tmp,sizeof(tmp));
    return result;
}
int proof_seal(void *ptr, const uint8_t *plaintext, size_t n,
               uint8_t keymsg[KEY_MAX],size_t *keyn,int *prekey,
               uint8_t payload[PAYLOAD_MAX],size_t *payloadn) {
    memset(keymsg,0,KEY_MAX);memset(payload,0,PAYLOAD_MAX);
    *keyn=0; *payloadn=0; *prekey=0;
    struct proof *ctx=ptr;
    if (!ctx || !ctx->pinned || !ctx->session.init || n>PLAIN_MAX ||
        !same(ctx->pin,ctx->session.remoteidentity,32)) return ERROR;
    struct proof tmp=*ctx;
    uint8_t input[PAYLOAD_MAX]={0},key[48]={0};
    struct omemo2KeyMessage message={0};
    memcpy(input,plaintext,n);
    int result=omemo2EncryptMessage(payload,key,input,n);
    if (!result) result=omemo2EncryptKey(&tmp.session,&message,key,sizeof(key));
    if (!result && message.n>KEY_MAX) result=ERROR;
    if (!result) {
        memcpy(keymsg,message.p,message.n); *keyn=message.n;
        *payloadn=n+omemo2GetMessagePadSize(n); *prekey=message.isprekey;
        memcpy(ctx,&tmp,sizeof(tmp));
    } else {memset(keymsg,0,KEY_MAX);memset(payload,0,PAYLOAD_MAX);}
    wipe(input,sizeof(input));wipe(key,sizeof(key));wipe(&message,sizeof(message));wipe(&tmp,sizeof(tmp));
    return result;
}
int proof_open(void *ptr, const uint8_t *keymsg,size_t keyn,int prekey,
               const uint8_t *payload,size_t payloadn,
               uint8_t output[PAYLOAD_MAX],size_t *outn) {
    memset(output,0,PAYLOAD_MAX); *outn=0;
    struct proof *ctx=ptr;
    if (!ctx || !ctx->pinned || !ctx->store.init || !keyn || keyn>KEY_MAX ||
        (prekey!=0 && prekey!=1) || !payloadn || payloadn>PAYLOAD_MAX || payloadn%16 ||
        (ctx->session.init && !same(ctx->pin,ctx->session.remoteidentity,32))) return ERROR;
    struct proof tmp=*ctx;
    uint8_t key[48]={0}; size_t kn=sizeof(key);
    int result=0;
    /* Established sessions may otherwise ignore the outer prekey identity.
     * Authenticate a prekey wrapper through the actual library with a fresh
     * session and isolated cache, then enforce its full Ed25519 pin. This costs
     * a second crypto operation and supports only this proof's bounded delay.
     * Prekeys remain until catch-up finishes, as required by the upstream API.
     */
    if (prekey) {
        struct proof check=*ctx;
        memset(&check.session,0,sizeof(check.session));
        wipe(check.skipped,sizeof(check.skipped));check.count=0;
        enter(&check);
        omemo2SetCallbacks(load_skipped,discard_probe_key,NULL);
        result=omemo2DecryptKey(&check.session,&check.store,key,&kn,true,keymsg,keyn);
        leave();
        if (!result && !same(check.session.remoteidentity,ctx->pin,32)) result=ERROR;
        wipe(&check,sizeof(check));wipe(key,sizeof(key));kn=sizeof(key);
    }
    if (!result) {
        enter(&tmp);
        result=omemo2DecryptKey(&tmp.session,&tmp.store,key,&kn,prekey,keymsg,keyn);
        leave();
    }
    if (!result && !same(tmp.session.remoteidentity,ctx->pin,32)) result=ERROR;
    size_t olen=0;
    if (!result) result=omemo2DecryptMessage(output,&olen,key,kn,payload,payloadn);
    if (!result && olen>PLAIN_MAX) result=ERROR;
    if (!result) {
        struct omemo2KeyMessage heartbeat={0};
        result=omemo2Heartbeat(&tmp.session,&tmp.store,&heartbeat);
        // This proof has no heartbeat routing. Stop before committing one.
        if (heartbeat.n) result=ERROR;
        wipe(&heartbeat,sizeof(heartbeat));
    }
    if (!result) {memcpy(ctx,&tmp,sizeof(tmp));*outn=olen;}
    else {memset(output,0,PAYLOAD_MAX);*outn=0;}
    wipe(key,sizeof(key));wipe(&tmp,sizeof(tmp));
    return result;
}
void proof_convert(uint8_t out[32],const uint8_t ed[32]) {
    uint8_t tmp[32];memcpy(tmp,ed,32);tmp[31]&=0x7f;
    omemoDriverEdPubToCvPub(out,tmp);wipe(tmp,sizeof(tmp));
}
static void put32(uint8_t **p,uint32_t v) {
    (*p)[0]=v>>24;(*p)[1]=v>>16;(*p)[2]=v>>8;(*p)[3]=v;*p+=4;
}
static int get32(const uint8_t **p,const uint8_t *end,uint32_t *out) {
    if ((size_t)(end-*p)<4) return ERROR;
    *out=(uint32_t)(*p)[0]<<24|(uint32_t)(*p)[1]<<16|(uint32_t)(*p)[2]<<8|(*p)[3];
    *p+=4;return 0;
}
int proof_export(const void *ptr,uint8_t output[SNAP_MAX],size_t *n) {
    memset(output,0,SNAP_MAX);*n=0;
    const struct proof *ctx=ptr;
    if (!ctx || !ctx->store.init || ctx->count>SKIP_MAX) return ERROR;
    size_t storen=omemo2GetSerializedStoreSize(&ctx->store);
    size_t sessionn=omemo2GetSerializedSessionSize(&ctx->session);
    size_t total=8+4+storen+4+sessionn+4+32+4+ctx->count*68;
    if (total>SNAP_MAX) return ERROR;
    uint8_t *p=output;
    memcpy(p,"F200S001",8);p+=8;
    put32(&p,storen);omemo2SerializeStore(p,&ctx->store);p+=storen;
    put32(&p,sessionn);omemo2SerializeSession(p,&ctx->session);p+=sessionn;
    put32(&p,ctx->pinned);memcpy(p,ctx->pin,32);p+=32;
    put32(&p,ctx->count);
    for (unsigned i=0;i<ctx->count;++i) {
        put32(&p,ctx->skipped[i].nr);
        memcpy(p,ctx->skipped[i].dh,32);p+=32;
        memcpy(p,ctx->skipped[i].mk,32);p+=32;
    }
    *n=p-output;return 0;
}
static int valid_state(const struct proof *ctx) {
    if (!ctx->store.init || ctx->pinned>1 || ctx->session.init<0 || ctx->session.init>3 ||
        ctx->count>SKIP_MAX || !ctx->store.cursignedprekey.id ||
        ctx->store.cursignedprekey.id>0x7fffffff || !ctx->store.pkcounter ||
        ctx->store.pkcounter>0x7fffffff || ctx->store.prevsignedprekey.id>0x7fffffff) return ERROR;
    for (unsigned i=0;i<OMEMO2_NUMPREKEYS;++i) {
        uint32_t id=ctx->store.prekeys[i].id;
        if (id>0x7fffffff) return ERROR;
        for (unsigned j=0;id && j<i;++j) if (ctx->store.prekeys[j].id==id) return ERROR;
    }
    if (ctx->session.init) {
        if (!ctx->pinned || !same(ctx->session.identity,ctx->store.identity.pub,32) ||
            !same(ctx->session.remoteidentity,ctx->pin,32) ||
            ctx->session.usedpk_id>0x7fffffff ||
            ctx->session.usedspk_id>0x7fffffff) return ERROR;
        if (ctx->session.init==1 && (!ctx->session.usedpk_id || !ctx->session.usedspk_id)) return ERROR;
    } else {
        const struct omemo2Session empty={0};
        if (!same(&ctx->session,&empty,sizeof(empty)) || ctx->count) return ERROR;
    }
    if (!ctx->pinned) {
        const uint8_t zero[32]={0};
        if (!same(ctx->pin,zero,32)) return ERROR;
    }
    for (unsigned i=0;i<ctx->count;++i)
        for (unsigned j=0;j<i;++j)
            if (ctx->skipped[i].nr==ctx->skipped[j].nr &&
                same(ctx->skipped[i].dh,ctx->skipped[j].dh,32)) return ERROR;
    return 0;
}
int proof_import(void *ptr,const uint8_t *snapshot,size_t n) {
    struct proof *ctx=ptr;
    if (!ctx || ctx->store.init || n>SNAP_MAX || n<8 || memcmp(snapshot,"F200S001",8)) return ERROR;
    struct proof tmp={0};
    const uint8_t *p=snapshot+8,*end=snapshot+n;
    uint32_t length=0,pinned=0,count=0;
    int result=ERROR;
    if (get32(&p,end,&length) || !length || length>(size_t)(end-p)) goto done;
    if (omemo2DeserializeStore(p,length,&tmp.store)) goto done;
    p+=length;
    if (get32(&p,end,&length) || !length || length>(size_t)(end-p)) goto done;
    if (omemo2DeserializeSession(p,length,&tmp.session)) goto done;
    p+=length;
    if (get32(&p,end,&pinned) || pinned>1 || (size_t)(end-p)<32) goto done;
    tmp.pinned=pinned;memcpy(tmp.pin,p,32);p+=32;
    if (get32(&p,end,&count) || count>SKIP_MAX || (size_t)(end-p)!=count*68) goto done;
    tmp.count=count;
    for (unsigned i=0;i<count;++i) {
        if (get32(&p,end,&tmp.skipped[i].nr)) goto done;
        memcpy(tmp.skipped[i].dh,p,32);p+=32;
        memcpy(tmp.skipped[i].mk,p,32);p+=32;
    }
    if (valid_state(&tmp)) goto done;
    /* Canonical re-encoding rejects ignored, duplicate, missing, or extra fields.
     * This is bounded framing of upstream serializers, never C object padding.
     * Snapshots carry private keys and are NOT encrypted/authenticated vaults.
     */
    {
        uint8_t canonical[SNAP_MAX]={0};size_t cn=0;
        if (!proof_export(&tmp,canonical,&cn) && cn==n && same(canonical,snapshot,n)) result=0;
        wipe(canonical,sizeof(canonical));
    }
    if (!result) memcpy(ctx,&tmp,sizeof(tmp));
done:
    wipe(&tmp,sizeof(tmp));return result;
}
