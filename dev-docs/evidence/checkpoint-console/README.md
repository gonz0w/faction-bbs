# Member-local checkpoint console software validation

Issue [#205](https://github.com/gonz0w/faction-bbs/issues/205), 2026-10-25.
The [saved record](results.json) covers the explicit default-off
[checkpoint console](../../../docs/checkpoint-console.md) and the unchanged v2
storage/delivery gates. This is software evidence, not a live XMPP inbox, an audit,
full protocol certification, physical storage test or supported-capacity claim.

```sh
python3 tools/check_checkpoint_console_evidence.py
```

This command checks immutable recorded bytes and outcomes. It does not run the
crypto, terminal tests or VM. The separate
[pre-edit baseline](../checkpoint-ui-baseline-manifest.json) preserves seven actual
client/CI inputs and all three complete #203 record identities. Earlier checkpoint,
interoperability and capacity records are unchanged.

## Actual observations

Both Darwin arm64 and an owned NetBSD 11 evbarm/aarch64 VM ran the final source:

- Rust 1.96 composed native suites: 72 passed per OS. Breakdown: client library17,
  client binary23, lab example5 and native adapter27. One intentionally ignored
  subprocess helper is invoked by the parent crash controls; it is not counted
  as an independent pass.
- New checkpoint console: 23 real PTY controls per OS, including hidden input and
  post-exit paste draining, deliberate upgrade, typed Ed25519 trust/rejection,
  bounded history/pending IDs, explicit retirement, true plain output/inert Unicode,
  backup/recovery/fresh enrollment, full/expired/revoked state and write-error reopen.
- Existing feature-off client: 13 v1 vault and eight picker controls per OS.
  The native command is rejected and creates no store when the feature is off.
- Host strict Rust1.96 fmt/Clippy passed. A separate reviewer independently checked
  the final Clippy/PTY source and the contract; reviews are not a crypto audit.

The guest had two cores, 1024 MiB configured RAM, base OpenSSL3.5.7, GCC12.5 and
Rust/Cargo1.96. The full native run used an unprivileged fixture account and cached,
hash-checked dependencies offline. No allocator, clock, NPF, resource default or
saved service was tuned. No new memory/capacity measurement is inferred from a
functional pass. Binary hashes/sizes in the record identify the tested artifacts;
they are not whole-client memory costs.

The record SHA-matches 264 public source/metadata/fixture/harness inputs, including
the exact [executed native runner](run-native.sh), plus four code-only intermediate
inputs. A capture does not claim every workspace file was compiled or executed.
The hostile-stream test feeds the actual product Filter byte-by-byte through a
PTY lab helper. It is **not** a real SSH visit or network-adversary certification.
The existing SSH/file paths were unchanged and retain their separate evidence.

## Reproduce the terminal controls

On a native build host with OpenSSL3 and Python3, build the explicit feature and
lab-only synthetic fixture, then run:

```sh
cargo +1.96.0 build -p faction-client --features native-mail \
  --bin faction-client --example checkpoint_ui_fixture --release --locked
python3 tools/probe_checkpoint_ui.py \
  --client target/release/faction-client \
  --fixture target/release/examples/checkpoint_ui_fixture
```

On NetBSD use the base OpenSSL paths described in the
[backend contract](../../native-mail-checkpoints.md) and the installed Python
executable, such as `python3.13`. The preserved runner shows the actual owned guest
paths and commands; it is evidence, not an installed service. Its individual
`.exit` results must all be zero; a `complete` marker alone is not a pass.
The fixture generates synthetic paired encrypted history/ciphertext only, opens
no listener and is not a mail transport or member-facing import command. Harness
results contain check names/counts, never terminal transcripts or generated keys.

## Failures and corrections retained

The actual initial PTY test failed before the new LOCAL entry existed. Later
review regressions observed leftover hidden input at the invoking shell and a
recovery display dismissed by pasted typeahead. Input flush fixes made both green.
Their exact executed [UI](../sources/205-console-red/checkpoint_ui-before-typeahead-fix.rs)
and [harness](../sources/205-console-red/probe-before-typeahead-fix.py) are preserved.

The issue clarified an incorrect initial UNSENT oracle before correcting it:
pending ciphertext cannot prove no earlier delivery occurred. The UI now labels
pending/retired operations with delivery unproven, and states that this UI attempts
no delivery. No protocol bound or rejection assertion was relaxed.

The first native UI run stopped in the test wrapper because `python3` was absent;
the guest exposes `python3.13`. The
[executed old harness](../sources/205-console-red/probe-before-netbsd-wrapper-fix.py)
is retained. Invoking the wrapper through `sys.executable` kept the NO_TYPEAHEAD
assertion intact. Strict Clippy then required an equivalent collapsed conditional;
the [pre-style Rust](../sources/205-console-red/checkpoint_ui-before-clippy.rs)
is retained. Final host/native suites reran on the corrected, matched sources.

New provenance/CI/evidence checks each had observed failing tests before their
implementation, including required feature-off/v1 compatibility evidence.
GitHub issue/PR comments own the delivery record; this report is immutable evidence.

## Teardown

The actual owned QEMU PID/overlay and controller were verified, shut down and
removed. The actual loopback administrative forward was verified closed. Owned
private keys/vaults/backups/test roots were removed and PTY children reaped. The
saved owner playground remained stopped: its saved files' hashes and disk
size/mtime matched before and after. Public Cargo caches were retained. No public
listener, provider contact, saved playground startup or deployment occurred.
