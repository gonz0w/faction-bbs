# Broader profile-v2 and original INN software evidence

133 workspace Rust tests pass on macOS ARM64 and native NetBSD 11 ARM64. New tests
cover UTF-8 B/Q subjects, malformed/decoded controls, MIME text encodings/defaults,
weekday/calendar rules, exact final bytes/newlines/whitespace, long logical lines
and root/recent reference pruning with a fixed 128-ancestor cap. Schema stays 6;
existing migration/restore/approval tests remain. Host format/clippy/build and
104 Python tooling tests pass; current main/standalone dependency inventories
select no new registry package/version/feature or JS/TS filenames.

Seven actual shared native UID/PTY controls retain subject defaults/overrides,
ordered/partial/full reading, output-drain acknowledgment and active revocation.
Local plain/ANSI reader and full console flows pass on both hosts; host owner
export/restore and worker flows pass. This is not a new dedicated SSH admission,
full SFTP, hardware, long-soak or 250-member capacity matrix.

Original INN 2.7.4 passes twelve controls on each host. Eight round trips preserve
canonical UTF-8 subjects, long text, final whitespace/newlines, MIME defaults,
base64 and Q words. Three invalid inputs are rejected by INN; one deliberately
invalid Date is accepted by INN but rejected by Faction. The separate Python
email/MIME parser also passes. These are offline news-software checks, not live
INN server/feed/provider interoperability or adoption.

The pinned source archive SHA-256 and each host's actual executable/wrapper/
libinn artifact hashes are recorded. Upstream source has no JS/TS filenames.
No upstream source/binary is bundled. The native clone initially lacked upstream's
Perl build tool; an official no-dependency perl-5.42.3 package was used only there.
A timeout exposed INN's early hostname lookup; the final harness forces the
synthetic documented hostname override, then an owned temporary loopback fixture
sends only 400 and verifies zero client commands. No real endpoints, credentials,
posting or public listener are involved. No host/NPF hardening activation.

For a 16,384-byte synthetic body, v2 emits 17,775 bytes. Both hosts measure 66,277
bytes of extra peak requested live allocation during parse and 32,256 during
serialization, above existing fixture allocations. The 1,000-iteration means are
single observations, not throughput budgets. Excludes stacks, allocator overhead,
RSS, SQLite, SSH, whole-service and concurrent-member costs. The measurement-only
allocator is never linked into product binaries.

All 100 product/theme/helper source inputs were hash-checked against the actual
native checkout before cleanup. Tenth snapshot freezes all 95 #113 inputs and
its unchanged original record; twenty-two earlier result identities stay fixed.
The disposable VM, reference package and synthetic state are removed; the owner's
stopped playground/checkpoint remain untouched. See [GitHub issue](https://github.com/gonz0w/faction-bbs/issues/103)
and [profile/reproduction](../../../docs/article-profile.md). Live #101/#104 and
Unicode composer #114 remain separate; hardening #106–#111 remains later.
