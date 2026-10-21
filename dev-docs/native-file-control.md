# Native Files control v1

Private implementation seam for #87 / ADR 0033; no general plugin/shell/RPC API.
The existing approved ANSI Files screen emits OSC `7770;FACTION1` carrying a
base64-encoded file-policy field frame. Plain SSH may ignore the control and keep
using the manual menu. A native client sends `native-v1 ED25519_BLOB NONCE` only
there; nonce is 16 random bytes in lowercase hex. The server validates the public
key and current approved session, enters its Files substate and replies
`hello,nonce,public_namespace`. No password, private key or role is negotiated.

Server frames are `ESC ] 7770;FACTION1;BASE64 BEL`. The decoded v1 field frame uses
`faction_files::encode/decode` without its outer u32 length: at most 4096 decoded
bytes, 24 fields and 1024 bytes/field. Client filters cap encoded payload at 5500
bytes and reject malformed/unknown controls; arbitrary OSC/DCS and terminal queries
are discarded. Rendered ordinary ANSI is restricted separately.

Requests are ASCII `@NATIVE1 NONCE SEQUENCE BASE64_FIELDS` plus newline/CR, at most
1024 bytes, 600 decoded request bytes / 800 encoded bytes. Sequence is positive u32,
strictly the connection's previous sequence plus one. Responses are
`row,nonce,sequence,ROW_FIELDS`, optionally `error,nonce,sequence,generic_error`,
then `end,nonce,sequence`; at most 20 rows. One in-flight client request, five-second
response deadline. Unknown operations are denied; stale nonce/sequence/current
membership fails the operation through the existing service recovery boundary.

| Request fields | Result row fields |
| --- | --- |
| `areas,after_id` | `area,id,slug,title,private,quota` |
| `list,area_id,after_object_id` | `file,id,name,bytes,sha256,uploader,description_excerpt` |
| `info,full_local_reference` | `info,id,name,bytes,sha256` |
| `upload,area_id,name,bytes,description` | `grant,upload,name,bytes,empty,grant_id` |
| `download,full_local_reference` | `grant,download,name,bytes,sha256,grant_id` |
| `cancel,grant_id` | `cancelled` |
| `back` | `bye,nonce,sequence`, followed by normal BBS menu output |

The native list includes published/currently accessible objects only. The server
selects the current member/epoch and reuses existing catalog/review/quota policy;
public-key binding never proves BBS membership. No host/path/DB/owner command is
accepted. Client-selected destinations remain in its private profile. After local
confirmation, base SSH opens the separate fixed SFTP subsystem with the bound key;
its single ready grant and each policy operation/chunk remain checked by the
existing file service. Never expose grant IDs as broad session authority.

The public `faction-file:NAMESPACE/OBJECT` reference has two 32-character lowercase
hex components, exactly 78 bytes. No hostname or query data. Resolve only against
the current board; possession grants no access. Maximum four recognized references
per room/chat message, per-reader permission redaction, and recheck at download.
Namespace remains stable across restore; private session binding does not.

Future adapters must preserve semantic scopes, version/length/deadline limits and
local confirmation. This protocol carries no mail/device secrets, ciphertext
attachment keys, home-identity proof or federation instruction. General plugin
loading and device-encrypted attachments remain separate work.
