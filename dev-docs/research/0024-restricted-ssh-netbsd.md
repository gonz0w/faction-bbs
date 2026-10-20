# 0024: Complete local application through restricted SSH on NetBSD

Date/retrieval: 2026-10-20. Issues #18/#19/#20/#23/#30/#31/#35.
Status: measured native software compatibility and bounded development gateway.
Decision: [ADR 0012](../decisions/0012-restricted-ssh-and-terminal-handoff.md).
Saved evidence: [sanitized results](results/024-netbsd-application.json).

## Question and environment

Can the real accounts/rooms/chat/menu/snapshot application compile and run natively
on the primary NetBSD target, and can ordinary SSH expose its member entry point
while a separate terminal OS identity cannot read the private database or invoke
owner recovery/export? This replaces synthetic-only SSH evidence with a current
application experiment; it does not close all access/security/resource issues.

Use NetBSD 11.0 evbarm ARM64, two vCPUs, 1024 MiB configured RAM, a disposable 12-GiB
copy-on-write virtual disk, QEMU/HVF on ARM64 Darwin. Stock image, native C compiler,
pkgsrc rust-bin 1.96.0 and Python 3.13. No host filesystem share or bridge; user-mode
NAT with administrative and member SSH forwards bound explicitly to host loopback.
The guest's member daemon binds one explicit private guest address; the checked-in
template defaults to loopback. Stock unrelated guest services make this unsuitable
as evidence of a minimal/hardened appliance. No physical SBC or overlay was tested.

Official image/kernel SHA-512 matched the signed release-wide manifest; GPG's valid
signature matched the recorded security-officer key. The key came from official
HTTPS, without independent web-of-trust validation. Exact URLs/hashes/fingerprint
are in results. The same HVF `vmt` probe workaround as prior research was needed:
kernel userconf `disable vmt`, `quit`. Pace serial input to avoid UART character
loss during this hypervisor's polled early console. This is a VM-specific finding.

## Predeclared oracles and actual results

1. Native offline Rust 1.96 complete workspace release build/tests under a non-root
   backend owner; native C/SQLite and application credential/storage/terminal code.
2. All four actual local PTY workflows: owner console, simultaneous chat, snapshot
   export/verify/restore and member worker without controlling `/dev/tty`.
3. Actual OpenSSH member sessions through the separate terminal UID and broker:
   real registration/pending/approval, ANSI/menu/rooms/chat/revoke/resize/disconnect.
4. Combined restricted-daemon/key/frontend denials for non-PTY, exec, blank exec,
   SFTP, direct/remote TCP, remote Unix forwarding, unknown key and owner export.
5. Positive native descriptor ownership transfer, malformed/excess/truncated rights,
   descriptor count unchanged and a proved-live worker before disconnect teardown.

All passed for the recorded sources. Saved JSON contains exact test/oracle counts,
names, binary sizes, versions, source hashes, cleanup and a two-member RSS/VSZ
sample. The negative-only handoff tests originally missed a native ABI mismatch;
the new independent positive transfer plus real sessions are required evidence.
Effective agent/X11/tunnel denials were inspected, not independently attacked over
the wire. Passwords/keys/actual endpoints/transcripts are excluded from results.

## NetBSD ancillary ABI finding

The first handoff delivered no usable terminal: native Rustix ancillary parsing
did not match the target's CMSG layout. Native C reported header size 12, payload
offset 16 and one-int space 32; pinned libc's NetBSD ARM64 `_ALIGNBYTES` uses int
alignment and computes offset 12/space 16. Observe this as a narrow reproduced
ABI incompatibility, not a claim that every dependency or NetBSD architecture fails.

`faction-unix` now calls the native header macros in a bounded C helper. Its small
Rust FFI ownership boundary is explicit; other application crates still forbid
unsafe. The positive test reads a known payload through the received descriptor,
drops that copy and proves the original still works. Local malformed probes send
1/2/80 descriptors, exercise rejection/overflow and preserve the broker's observed
descriptor count. No registry patch, copied upstream code or external service is
required. Track this under #31/#23 until an upstream candidate passes the same tests.

## Reproduction

Use a **fresh disposable guest only**, never an existing board. Follow the signed
image/native build steps in [the earlier VM guide](../../experiments/023-netbsd-dependencies/README.md),
but bundle the **workspace** source/lock plus `cargo vendor --locked` output outside
the checkout. Write Cargo source replacement config for that vendor directory,
then build inside NetBSD with `cargo test --workspace --release --frozen -j 1` and
`cargo build --workspace --release --frozen -j 1`. No cross-compilation substitutes
for this evidence. Native C and complete Rust std/toolchain must exist on the target.

Use [the owner/SSH setup guide](../../docs/restricted-ssh.md) for dedicated identities,
root-controlled binaries/home/keys, private database/socket and rendered daemon
policy. The test fixture additionally provides a separately keyed root control
daemon inside the private VM and `/usr/pkg/bin/python3.13`. Confirm both host keys
through the trusted serial/admin path. Do not install that control path publicly.
Create a root-owned `0600` `/etc/faction-disposable-test` marker and leave the test
board/database/socket absent. The driver refuses an existing database; repeated
failed runs require deliberate inspection/reset of **only** that disposable fixture.

Run all local smokes as the backend owner, then from the developer host:

```sh
python3 tools/check_restricted_ssh.py --disposable-fixture \
  --host "$TEST_HOST" --port "$TEST_MEMBER_PORT" --admin-port "$TEST_ADMIN_PORT" \
  --identity "$TEST_MEMBER_KEY" --admin-identity "$TEST_ADMIN_KEY" \
  --unknown-identity "$TEST_UNKNOWN_KEY" --known-hosts "$TEST_KNOWN_HOSTS" \
  --output "$TEST_RESULT_FILE"
```

The driver generates private BBS credentials, bootstraps only the fresh synthetic
board, starts its broker and tests real login/registration/approval. Fixed paths
and privileged controls are explicitly test-fixture tooling, not product API.
Default launch throttles stay enabled; workflow batches wait for refill. Use a
known live PID for fstat and observe the worker count before and after disconnect.
NetBSD ps needs separate `-o` options when suppressing multiple headings; exact
name matching may include the executable path. These test-harness corrections
prevent missing observations from being counted as successful cleanup.

Shut down the guest, stop only this task's QEMU/serial control processes, remove
forwards and delete generated keys/disks/logs after extracting sanitized evidence.
The saved run records completed cleanup. No developer-host SSH config is changed.

## Resource observations and limits

This is a two-member compatibility sample, **not 100/250-member load validation**.
Read the saved per-process RSS/VSZ values with caution: RSS contains shared pages,
VSZ is not resident memory, and this sample excludes OpenSSH/OS cache, transient KDF
peaks, queue pressure and full host accounting. The default gate remains eight
workers. Each member adds a frontend, trusted application worker and SSH processes;
measure a shared-runtime/IPC alternative before scaling this architecture blindly.

The code bounds handoff/launch concurrency and receive/prompt/session lifetimes.
It has no measured production output deadline, per-account/IP fair queue, native
power-loss/disk-full/broker-crash campaign or complete process sandbox. Root and
trusted worker compromise remain outside the scoped pass results. Remaining account,
room/file lifecycle and maintained service/appliance operations remain open.

## Sources and confidence

Primary sources retrieved 2026-10-20: [NetBSD sshd_config](https://man.netbsd.org/sshd_config.5),
[sshd](https://man.netbsd.org/sshd.8), [recvmsg](https://man.netbsd.org/recvmsg.2),
[QEMU ARM guide](https://wiki.netbsd.org/ports/evbarm/qemu_arm/),
[release manifest](https://cdn.netbsd.org/pub/NetBSD/security/hashes/NetBSD-11.0_hashes.asc),
[security-officer key](https://cdn.netbsd.org/pub/NetBSD/security/PGP/security-officer@netbsd.org.asc),
the guest's native headers and exact pinned libc/rustix source trees. Saved source
hashes and checked-in executable tests tie the claims to this revision.

High confidence in these native software oracles; low confidence in unmeasured
capacity, hostile-code confinement, other NetBSD ports, physical hardware or full
production security. Native/application compatibility now has direct evidence;
that does not select a production backend or supported appliance by implication.

## Managed lifecycle follow-up

After #52 merged, [report 0026](0026-service-lifecycle-load.md) adds the fresh native
service/load/failure campaign. Saved 024 application evidence was refreshed from
that same final code with all 54 Rust/four PTY/20 SSH oracles passing. Its JSON
now records the updated binary/process observations and source hashes; the
original #52 merge retains the original observations. Broad unperformed gates
above remain except the bounded lifecycle/broker-loss cases explicitly covered
by 0026. This is still eight-worker software evidence, not production capacity.
