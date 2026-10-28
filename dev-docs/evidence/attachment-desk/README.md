# Native private attachment desk software evidence

This record covers #263 and #264, the explicit default-off `network-mail` console
on one macOS software host. The original #88 interoperability/provider/NetBSD
requirements remain open. It is not an emulator/font, SSH trace, deployed home,
cryptographic device-attestation, hardware, security-audit or capacity certificate.

## Observed terminal journey

[The final host result](host-result.json) records twelve actual production PTY and
owned two-home TLS controls in 18.072 seconds, including fixture creation and
unlocks. It transferred exactly 65,536 plaintext bytes. Classic normal/40-column
and explicit Unicode paths were exercised. The actual original source snapshot
was queued even after its file changed at confirmation; retries then worked with
the source deleted. A real durable chunk reply was lost; that action stopped and
an explicit reopened operation resumed with one hidden password. After the sender
went offline, the recipient used its own pinned home to fetch, verify and confirm
a private save. Declined route/queue/save, queued typeahead, no overwrite, inert
bidi metadata and Ctrl-C/D lock/TTY restoration controls passed. Server files and
storage IQs lacked the selected private metadata, keys and plaintext pattern.

These endpoint counters are fixture observations, not member-side parser telemetry
or whole-call bounds. Existing SDK/session bounds remain separate. Source, binary
and selected verification identities are in [inputs](inputs.json); exact selected
source bytes are under `sources/`. The unchanged parked-console caller is included;
this is not a fresh same-SSH/NetBSD tracing observation. Dependencies remain on the
existing locked graph. Prior TLS/source records are unchanged.

## Tests, limitations and retention

[Verification facts](verification.json) distinguish actual test-first failures,
recovery/integrity regressions, setup and intermediate failures, and the exact cache
inventory correction. The five library and three binary ignored entries are existing
subprocess helpers; their parent controls invoke the relevant helpers. A prior
expanded pass is not substituted for the final frozen-source replay. Hosted checks
and merge remain separate integration gates.

[Separate review facts](review.json) describe a nonauthor examination, not independent
human assurance. [Private retention](private-retention.json) identifies the verified
owner-only archive of original captures, source variants and executed binaries.
Raw logs, transcripts, runtime vaults, credentials and TLS keys are not public.
Fixture PTYs/processes and TLS homes were closed/reaped and their ephemeral
runtime directories removed. Saved source/capture roots are retained privately
until verified archival and cleanup; metadata-only final checks remain private.
The owner playground stayed stopped and intact.

Reproduce with the `network-mail` client, `mail_blob_fixture` and
`mail_reply_fixture`, built with Rust 1.96.0 and the existing OpenSSL 3 setup. Create
an owned mode-0700 directory containing `OWNED` with exactly
`Owned native attachment console proof` followed by a newline, then run:

```sh
python3 tools/probe_mail_attachments.py --client /absolute/path/faction-client \
  --fixture /absolute/path/mail_blob_fixture \
  --reply-fixture /absolute/path/mail_reply_fixture \
  --owned-root /absolute/owned/proof-root --output /absolute/owned/proof-root/result.json
```

Use only synthetic owned loopback homes. The tool retains private PTY captures
under that root and removes temporary runtime keys/vaults after closing children.
The [member guide](../../../docs/private-attachments.md) explains format opt-in,
separate spool backup/expiry/orphans, explicit actions and current device limits.
