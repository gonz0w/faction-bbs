# Native XMPP transport qualification

Date: 2026-10-25. [#208](https://github.com/gonz0w/faction-bbs/issues/208), finite
child of #90. Result: qualify a narrow tokio-xmpp research profile for deliberate
adoption in [#209](https://github.com/gonz0w/faction-bbs/issues/209). Root product
Cargo, native OMEMO, encrypted vault formats and ordinary BBS operation are unchanged.
The [contract](../native-xmpp-transport.md) is authoritative for the tested profile;
[saved observations](../evidence/xmpp-transport/README.md) distinguish software passes,
rejected builds, actual costs and cleanup. No live mailbox is shipped.

## Two candidates

| Candidate | Pinned source/terms | Outcome |
| --- | --- | --- |
| tokio-xmpp 6.0.0 | xmpp-rs `1710a6f99edd18cd926a2dc5d14ec532a2a6ad72`; MPL-2.0. Crate archive SHA256 `8921a165ab1eefd447cb1661423c582ac77b9c6960e123a71c8ad9cebc40b026`. | Existing custom streams/typed XML/SASL plus fixed binding-IQ glue permit a bounded wrapper. Qualified with a tested incremental admission guard and explicit lifecycle; stock defaults are not adopted. |
| libstrophe 0.14.0 | `7c2e78937d1f8614339bfa88bdcc21bb406deadd`; MIT OR GPL-3.0-only, MIT option considered. Canonical inspected archive SHA256 `adeb9e673d7f8b8cd20a2437cc220a8de581abf6e46594896b622e3e0dfa5c1f`. | Stock Expat/event-loop path constructs full stanzas before public callbacks and has no selected pre-parser byte/depth admission hook. Reject for this contract; not built or runtime-cost measured. |

The C option's smaller source tree does not satisfy the parser boundary by itself.
Its 4096-byte receive buffer bounds a read, not a whole stanza. Its TLS key-log
configuration would also need an explicit local policy. The Rust wrapper uses
existing parsers and library authentication, not custom cryptography or another
handwritten protocol implementation.

## Protocol and lifecycle

The selected experiment uses direct TLS, explicit trust anchors/hostname/full DER
pin, then PLAIN as defined in RFC 4616. PLAIN itself provides no integrity or
confidentiality; it is qualified only inside the verified TLS channel. Tests observe
TLS 1.2 on macOS native TLS and TLS 1.2/1.3 on NetBSD OpenSSL. STARTTLS, SCRAM, discovery, components, web transports and automatic
reconnect are not qualified. Do not infer full current RFC/XEP conformance.

Use lower-level stream/auth APIs. The general Client's reconnect and logging
behavior is inappropriate for this narrow boundary without deliberate control.
The stock parser has a token bound but no complete application stanza/depth budget.
The wrapper admits bounded bytes through incremental rxml before DOM construction,
retains lifetime counters across one authenticated restart, and conservatively
charges root whitespace to the next interval. Whole-connection and cooperative
wall-clock limits bound dribbling/outage; generated regular local CA files are the
qualified input, not arbitrary blocking local paths.

TLS home-account authentication, server certificate pins, typed OMEMO Ed25519
trust, local keystore unlock and visited-board approval remain different authorities.
The probe only transports opaque synthetic bytes. #209 must validate the actual
OMEMO-2 Wire/SCE profile and couple it to atomic checkpoint/outbox operations.
No transport acknowledgement may silently retire an ambiguous operation.

## Source, platform and maintenance cost

The [inventory](../dependencies/xmpp-transport-research.json) covers 127 selected
registry packages on ARM64 NetBSD and x86_64 Linux, and 125 on ARM64 macOS, including features,
licenses and registry checksums. Native TLS uses system OpenSSL on NetBSD/Linux
and the native platform backend on macOS. Selected source and archive bytes pass
the no-JS check; upstream CI/workflows are not selected or executed. Unselected
WebAssembly/other-target lock/index metadata is not a source/runtime adoption.
The native fixture did not download JavaScript sources to satisfy directory-vendor
resolution; it used verified Cargo source/index cache offline.

The larger Rust dependency/build graph is a concrete cost. Fresh release compiler
attempts at 1 GiB failed at two jobs and serially with rustc SIGKILL. Kernel
out-of-swap diagnostics were captured for the two-job phase only; no later serial
phase diagnostic is invented. A2GiB builder
completed the unchanged dependency code. After the guard review fix, the cached
incremental rebuild/tests and actual transport/resource cases ran at 1 GiB. This is
not a claim that a fresh1GiB source build works. Individual process RSS and artifact
sizes do not include a whole BBS/home server, compiler, physical SBC or 250 members.

Update deliberately: preserve lock/checksums/notices, review the selected source
and local tool/CI graph, re-run source matching and native positive/negative tests,
and retain old failures/results. Never refresh old hashes to a newer library.
Product adoption remains a separate reviewed change; no online Python helper or
mandatory external service is introduced.

## Primary sources

Retrieved 2026-10-25. Source observations are not reproduced vulnerability reports
or a security audit.

- tokio-xmpp [manifest](https://gitlab.com/xmpp-rs/xmpp-rs/-/blob/1710a6f99edd18cd926a2dc5d14ec532a2a6ad72/tokio-xmpp/Cargo.toml),
  [connection API](https://docs.rs/tokio-xmpp/6.0.0/tokio_xmpp/connect/trait.ServerConnector.html),
  [authentication](https://gitlab.com/xmpp-rs/xmpp-rs/-/blob/1710a6f99edd18cd926a2dc5d14ec532a2a6ad72/tokio-xmpp/src/client/login.rs),
  [stream lifecycle](https://gitlab.com/xmpp-rs/xmpp-rs/-/blob/1710a6f99edd18cd926a2dc5d14ec532a2a6ad72/tokio-xmpp/src/stanzastream/mod.rs),
  [XML reader](https://gitlab.com/xmpp-rs/xmpp-rs/-/blob/1710a6f99edd18cd926a2dc5d14ec532a2a6ad72/tokio-xmpp/src/xmlstream/common.rs)
  and [message extension representation](https://gitlab.com/xmpp-rs/xmpp-rs/-/blob/1710a6f99edd18cd926a2dc5d14ec532a2a6ad72/parsers/src/message.rs).
- libstrophe [release](https://github.com/strophe/libstrophe/commit/7c2e78937d1f8614339bfa88bdcc21bb406deadd),
  [Expat parser](https://github.com/strophe/libstrophe/blob/7c2e78937d1f8614339bfa88bdcc21bb406deadd/src/parser_expat.c),
  [event path](https://github.com/strophe/libstrophe/blob/7c2e78937d1f8614339bfa88bdcc21bb406deadd/src/event.c)
  and [TLS source](https://github.com/strophe/libstrophe/blob/7c2e78937d1f8614339bfa88bdcc21bb406deadd/src/tls_openssl.c).
- [rxml 0.14 parser source](https://docs.rs/crate/rxml/0.14.0/source/src/parser/common.rs),
  [RFC 6120](https://www.rfc-editor.org/rfc/rfc6120.html),
  [RFC 4616](https://www.rfc-editor.org/rfc/rfc4616.html),
  [RFC 7590](https://www.rfc-editor.org/rfc/rfc7590.html) and
  [NetBSD resource units](https://man.netbsd.org/getrusage.2).
