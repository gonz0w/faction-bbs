# Optional native XMPP transport

`Home::new(endpoint, domain, full_jid, ca_pem, leaf_pin)` checks the numeric
endpoint, exact normalized account/resource identity and preloaded CA bytes.
The native client owns trusted private regular-file validation and hidden local
credential input. `connect(&home, password)` makes one direct-TLS attempt, verifies
the explicit CA, name and SHA256 DER leaf pin before SASL PLAIN, then binds the
exact configured full JID. The caller keeps credentials in zeroizing memory;
qualified upstream SASL temporarily copies ordinary strings in memory.

Use a current-thread Tokio runtime. `Session::send(&Element)` and `next()` share
the original absolute cooperative ten-second connection deadline. Dropping the
session or a pending operation closes its socket; a cancelled or failed operation
cannot resume a partial stream. No background task, DNS discovery or reconnect
runs. `Error` contains local classifications only. TLS key-log environment variables
are refused, and the qualified protocol logging is disabled at compile time.

The copied #208 guard precedes DOM construction: 65536 physical XML bytes across
both stream epochs, conservative 16384-byte intervals, depth 32 including the
stream root, and rejection on event 256. One trusted SASL restart preserves lifetime
counters and unconsumed-byte replay. Outgoing DOMs are bounded before encoding;
the exact qualified XSO serializer checks standalone and stream-context output
against 16384 bytes before a network write. Namespace overhead may make admission
conservative. TLS/socket buffers and provider work remain outside XML counters;
cooperative timeouts do not preempt blocking native code.

The higher layer owns bounded IQ correlation, the OMEMO codec, verified peer-device
trust, durable protocol checkpoints and outbox decisions. XML parsing and successful
socket writes do not authenticate mail content or prove delivery. This crate owns
no vault keys, history, mailbox cursor, durable queue or peer receipts.

The dependencies retain their original licenses, including MPL-2.0 for the
unmodified XMPP crates. Package pins and compile-disabled logging are adopted from
`experiments/208-xmpp-transport`; the guard and guard controls identify their source.
Changing that profile requires the repository's graph/source/license review.

Run the standalone controls with:

```sh
cargo +1.96.0 test --manifest-path crates/faction-xmpp/Cargo.toml --all-targets --locked
cargo +1.96.0 clippy --manifest-path crates/faction-xmpp/Cargo.toml --all-targets --locked -- -D warnings
```

`examples/transport_fixture.rs` is a lab-only compatibility driver for
`tools/probe_home_transport.py`. Its fixed synthetic identity and ciphertext-shaped
payload exercise the real product transport. They do not demonstrate the product
OMEMO codec or an encrypted two-home exchange. Trusted generated fixture CA setup
precedes its timer; it does not qualify arbitrary blocking local file inputs.
