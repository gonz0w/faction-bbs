# Native NetBSD encrypted attachment qualification

Issues #266 and #267 qualify the existing attachment libraries, transfer and
production console on **NetBSD 11.0 ARM64 software**, using two runtime CPUs and
1,054,990,336 observed physical bytes (a configured 1 GiB VM). This closes the
fresh native-platform gap for these paths. Provider/device-attestation deployment
and independent modern-client interoperability remain under #88.

## Actual observations

The native build used Rust/Cargo 1.96.0, existing base OpenSSL 3.5.7 and Python
3.13, two compiler jobs, two CPUs and a separate 2 GiB compiler envelope. It
completed in 99.846 seconds. The guest was shut down and restarted with 1 GiB
before runtime tests; build resources are not the runtime or server budget.

[Native tests](tests.json) record 299 Rust passes across 32 test entries and ten
existing subprocess helpers invoked by parent controls, plus 21 collector unit
tests. The unchanged [1 MiB transfer proof](transfer-result.json) passed in 84.43
seconds: lost durable reply, 64 resumed upload and 64 fetch calls, between-chunk
revocation, sender offline, route/selection denials, integrity and explicit save/no
overwrite controls. [The production console](console-result.json) passed all 12
classic/compact/Unicode PTY controls in 7.288 seconds, including fresh opt-in,
typeahead refusal, reviewed snapshot mutation, source removal/reopen/retry, own-home
fetch, inert metadata, save/cancel/no overwrite and Ctrl-C/D lock/TTY restoration.
Those times include fixture work, subprocesses and unlocks; they are not latency,
throughput, parser telemetry, memory-headroom or multi-member capacity budgets.

[The actual local native envelope](native-envelope.json) binds the exact selected
source, three ELF binaries and result bytes to local uname/sysctl/file observations.
The existing probe's `netbsd_qualification=False` remains unchanged. This separate
record establishes the narrower native execution context; it does not make the
collector an attestation that arbitrary supplied results really ran. The retained
native command captures and separate review establish this observation's execution.
Both logical homes, handler and member clients ran inside one guest on loopback;
this does not prove independent-machine failure/compromise isolation.

## Identities, limits and reproduction

[Build identities](build-identities.json) contain 337 tracked matching build/test
inputs, compiler versions, binary digests and eight `ldd`-reported system libraries.
The list is a captured input set, not a claim that each file was individually
compiled, a hermetic build attestation or a complete runtime-loader attestation.
The base source is preserved in [commit c685282](https://github.com/gonz0w/faction-bbs/tree/c685282ae727d8cc2327f66a690e4b0c70c640ff).
The exact 21 shared source objects are reused from the earlier immutable attachment
desk selection; new collector/test bytes are under `sources/`. No prior record,
source hash, dependency inventory, product source, SDK/UI assertion or cap changed.

[The native reproduction instructions](REPRODUCE.md) give the actual build/test
commands, materialized binary paths, unchanged transfer/console invocations,
manifest construction and the collector invocation. They create a fresh private
observation; the original result identities remain unchanged. Native compilation
is required by the existing FFI build. Collector unit tests use explicit synthetic
observations; the actual host CLI refuses native qualification. No CLI platform or
pretend override exists. Dependencies resolved with `--locked --offline`; no host
installation or new package adoption was needed.

[Qualification facts](qualification.json) distinguish genuine test-first tooling
failures and setup/refusal variants. A source permission refusal was fixed by
narrowing the staged public files, preserving content, binary and result digests.
Earlier VM/PATH/index/library setup failures are not claimed as behavioral RED or
passing qualification. [Private retention](retention.json) identifies verified
owner-only original captures and binaries. Generated transfer keys/vaults were
removed by the existing probes. Raw SSH settings, owner data, keys, transcripts
and complete VM images are excluded from public evidence.

QEMU was configured with restricted user networking and one explicit loopback
admin forward, following its [documented network semantics](https://www.qemu.org/docs/master/system/invocation.html).
No saved playground instance or public provider was started. [Teardown facts](teardown.json)
record stopped test processes, closed forwarding, unchanged selected original
saved assets and removal of the disposable disk, kernel/key copies and staging
archives after verified private capture archival.
This is software qualification, with no physical SBC, same-SSH trace, external
client interoperability, deployed-provider/device-attestation, security-audit or
250-user support claim. See the [member guide](../../../docs/private-attachments.md).
