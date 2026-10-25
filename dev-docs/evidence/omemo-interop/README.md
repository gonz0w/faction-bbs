# Saved two-way OMEMO qualification

[Results](results.json) record actual host and disposable NetBSD execution for
[#200](https://github.com/gonz0w/faction-bbs/issues/200) and
[#202](https://github.com/gonz0w/faction-bbs/issues/202). Read the
[research report](../../research/0054-omemo-interop-netbsd.md) and
[reproduction instructions](../../../experiments/200-omemo-interop/README.md).

The unchanged baseline is rejected in both initiation directions on both routes.
Only the explicitly patched candidate passes 92 unchanged-reference controls per
route. A reversed-reference-AD diagnostic is saved separately and never qualifies.
Current common baseline/corrected sources each pass 17 Rust/FFI tests on each OS.
This is generated-library/FFI interoperability, not product mail, complete XEP
conformance, a cryptographic audit, live federation or a capacity claim.

`execution_sources` pins the final source-matched release proof and reference.
`frozen_sources` pins thirty actual intermediate files under
[omemo200](../sources/omemo200), including baseline/role-red sources, the rejected
field-16 assertion patch and pre-style-refactor code. These are historical proof
inputs, not runnable product implementations or a second work tracker. The
[upstream notice](../../../experiments/200-omemo-interop/NOTICE.md) applies to the
retained Picomemo-derived patch material. No private test keys/stores are retained.
Original archive, corrected map, reference wheel licenses/hashes and raw RSS units
are in the record. Existing #199 and older records remain unchanged.

```sh
python3 tools/check_omemo_interop_evidence.py
python3 -m unittest discover -s tools -p 'test_omemo_interop*.py'
```

The offline guard rejects baseline promotion, modified-reference qualification,
missing reciprocal/restart controls, source tampering, altered profiles and false
teardown. Its independent record digest also rejects measurement rewrites. It checks saved identity and semantics, not fresh cryptography or VM
execution. It cannot certify a source audit, full client cost or genuine execution
by itself. Future edits must preserve these executed bytes under the repository's
[evidence contract](../README.md), never update old measurements to new source.

The marked disposable VM and host synthetic keys/runtime/builds are removed;
loopback ports are closed. The owner playground stays stopped and unchanged.
[#203](https://github.com/gonz0w/faction-bbs/issues/203) owns the separate encrypted
transactional store and source-adoption gate. Keep parent #90 open.
