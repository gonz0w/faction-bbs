# Sustained 250-member activity and bounded abuse

Status: completed bounded software experiment; admission fairness/work and CPU-budget gates remain open.
Date: 2026-10-21 UTC
Issues: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #28/#23/#19/#20 and [#66](https://github.com/gonz0w/faction-bbs/issues/66).
Authority: owner requested sustained-load/abuse work after merging #65. No supported capacity decision.
Stopping condition: two complete fresh-boot 15-minute active intervals plus the matched prior matrix, bounded abuse/recovery probes, complete evidence/cleanup and a focused PR.

## Question and declared gates

Does the unchanged selected Rust/base-SSH/isolated shared service retain useful
250-member behavior and the original resource guard during sustained activity and
bounded malicious input/preauthentication work? Keep all measured historical
sources/results unchanged. No physical SBC, new resident server, weaker credentials,
JavaScript, automatic host tuning or default-cap increase.

[Plan 0023](../plans/0023-sustained-abuse.md) declares the workload before execution:
15 minutes at 250 distinct approved local accounts, rotating chat sender with a
one-broadcast/second target, delivery confirmation to every connected member;
30-second room/menu/read/post cycles; 60-second disconnect/fresh real credentials;
a 2,048-invalid-key menu burst with a 249-peer live-chat witness; and eight silent
preauthentication peers plus at least 16 excess offers during the final three
minutes. During that final phase reconnects/UI cycles pause so the unchanged native
source penalties can expire without listener/registry reset. Actual activity and
latency are reported rather than treating the target rate as a measured rate.

The complete prior 32/100/200/250 matrix remains: real member/owner credentials,
full UI, exact live process limits, 0/19/38-MiB touched-page controls both idle and
loaded, backend stop, reaping and a fresh login. The loaded controls run **after**
the active interval to expose retained growth. The guard stays free + max(0,
clean_file - wired) >=128 MiB, zero swap, processes+64<maxproc and files+128<maxfiles.
A native one-second sampler spans the active interval; short credential/control
peaks retain the existing 10-ms/150-sample streams. These are sampled predicates,
not continuously maintained reserves. Whole-host CPU includes all OS services;
RSS sums repeat shared mappings and are not unique/physical application budgets.

## Alternatives and decision scope

| Alternative | Gate and disposition |
| --- | --- |
| Raise defaults after short idle tests | Insufficient active/abuse/soak evidence; keep eight. |
| Reduce credential cost or reserve; omit base SSH/time daemon | Violates equivalence/resource gates; rejected. |
| Add a new SSH server/proxy | New security/dependency and memory costs; no adoption here. |
| Exercise unchanged installed profile with bounded external drivers | Selected experiment; no product binary/schema/default change. |
| Call 15 minutes a long-duration soak | Unsupported; longer runs, broader mixes and complete budgets remain #28. |

No numerical winner score is useful: this is a mandatory behavior/resource screen,
not a new runtime ranking. A guard or functional failure prevents a passing claim;
retain all executed trials and exact inputs. Production adoption still requires
independent authority review and maintained host/image/timekeeping policy.

## Reproduction and provenance

Use a fresh disposable signed NetBSD 11 ARM64 QEMU/HVF VM with 2 vCPU/configured
1 GiB, zero swap and the explicit prior compact host+sealed allocator/pageable-NTP
research profile. Retain all base services. Root-owned experimental process/file
limits are 2048/16384 only for the workload and restore afterward. Start from 1,000
independently salted approved members plus sysop, 50 rooms and 100,000 ~1-KiB posts.
The preceding UI oracle leaves bounded synthetic fixture rows: the first trial
starts with 1,002 accounts, 52 rooms and 100,004 non-load posts. Those rows and
first-trial load posts remain across the second boot; this is a retained-data
replication, not an identical database snapshot. Record stopped counts/integrity
and restore the exact 1,001/50/100,000 baseline after the campaigns.
A shared synthetic SSH invitation key is not a large-keyring support claim.

Build/test the locked workspace and native helpers in separate target directories;
install the selected isolation and key-admission profile according to their guides.
Run the native 14 admission, 6 channel/restart and 29 UI checks before measurement.
Configure 250 reservations/250-ms key refill and 250 sessions/100-ms application
refill explicitly. Keep two kernel credential slots and both permit lifetimes,
unchanged Argon2, process-birth liveness, forced-command and UID/confinement gates.
All execution-source, installed ELF/library/helper/collector and client identities
are saved in the result; actual native bytes are compared with the frozen map.

```sh
python3 tools/check_sustained_host.py --disposable-fixture \
  --host "$PRIVATE_FIXTURE" --port "$MEMBER_PORT" --admin-port "$ADMIN_PORT" \
  --identity "$INVITATION_KEY" --admin-identity "$ADMIN_KEY" \
  --known-hosts "$VERIFIED_HOSTS" --secret-file "$PRIVATE_SYNTHETIC_SECRET" \
  --model shared --allocator compact --output "$PRIVATE_RESULT"
```

[The driver](../../tools/check_sustained_scaling.py) retains the matched matrix and
calls [bounded activity](../../tools/sustained_workload.py). The native collector
still accounts every dedicated SSH descendant and required frontend; no daemon
is added. [The long-stream reader](../../tools/sustained_stream.py) aggregates
process rows on the **developer test host**, avoiding unbounded host transcript
retention. Its 128-MiB wire-data/1,000-event ceiling is a client-parser bound, not a
server memory allowance. Private fixture endpoints, credentials, keys, messages,
raw logs and disk/forward artifacts are never published.

Separately run `tools/check_unsigned_keys.py` only on the marked quiescent member
fixture. The research-only client is Paramiko 4.0.0 plus its recorded resolved
Python/native dependencies in a private venv; it is not a Faction runtime/server
or required repository-check dependency. The probe verifies the trusted serial
host key, sends at most 64 unsigned known-key requests within 20 seconds and never
signs or obtains BBS authority. It deliberately uses a private AuthHandler API;
record installed module/native-file hashes for reproducibility.

Run `tools/check_source_fairness.py` with the same private admin/member arguments
after the load campaigns: eight silent peers, native grace expiry, a legitimate
same-source key attempt and timed recovery without listener/registry resets or
penalty changes. This is one synthetic NAT source, not broad address fairness.

The separate `experiments/055-sustained/cpu_accounting.c` diagnostic runs only
**after** the load campaigns: five seconds each sleeping, continuously busy and
30-ms busy/20-ms sleeping. It retains raw aggregate/per-CPU state ticks,
monotonic wall time, process/thread CPU clocks and getrusage deltas. The active
interval's statistical tick estimate and process runtime accounting disagreed;
retain both, calibrate independently and withhold CPU-throughput/budget acceptance.
No additional compiler/busy control runs during a measured 250-member interval.

## Observations

Both fresh-boot campaigns completed the 900-second active loop, all matched
32/100/200/250 stages, every original memory/credential/UI/limit control and fresh
recovery. The native long stream spans 905 samples at a requested one-second
interval, including a short collector tail; `active_seconds` includes that tail.
The final broadcast timestamps establish the actual activity interval separately.
These two runs are longer software observations, not a long-duration soak.

| Boot | Broadcasts / confirmed peer deliveries | Minimum active headroom | Minimum stress/control headroom | Final pre-stop headroom | Loaded stop |
| --- | --- | --- | --- | --- | --- |
| 1 | 851 / 211,899 | 154.10 MiB | 134.33 MiB | 175.07 MiB | 1.352 s |
| 2 | 851 / 211,899 | 152.97 MiB | 133.13 MiB | 173.93 MiB | 1.794 s |

Total **423,798 confirmed peer deliveries** from 1,702 broadcasts,
plus 23 room/read/post/input-burst cycles and 11 fresh real-member logins per run.
Every round confirms 250 distinct approved members. The native `recipients=250`
field counts matching console observations, including the sender. The matcher
cannot distinguish the sender's draft echo from its history display; exclude
that console from confirmed peer-delivery counts. Thus 1,702 broadcasts produce
**425,500 console-marker observations** and **423,798 non-sender receipts**. No
protocol-level delivery acknowledgement or stronger self-delivery claim is made. Each 2,048-invalid-key burst
has a separate 249-peer live-chat witness. All original sampled guards pass,
including the post-active 38-MiB control; zero swap, original application hard
ceilings and all SSH/frontend/base-service costs remain. Stop/reaping/fresh login
pass. The result retains actual rates, per-round latencies, CPU readings and
all native samples rather than reporting the target as achieved throughput.

The loaded silent cohort reaches eight; excess drops are 23/24, including native
probabilistic drops while filling the cohort and at least 16 excess offers.
Established members keep receiving broadcasts during grace and after expiry.
No listener or registry reset is used to clear the pressure. The full native
transport ledger remains in the resource predicate. Silent preauthentication
overlaps ordinary activity; the strongest 38-MiB control runs after those peers
expire. Simultaneous worst-case preauthentication, credential and output pressure
remains an explicit #28 adoption gate.

The separate quiescent known-key probe receives 64 PK_OK responses in about two
seconds: one reservation, 63 reuses, zero signatures/authentication and release
on disconnect. This confirms the existing open lookup-amplification gate;
[#66](https://github.com/gonz0w/faction-bbs/issues/66) owns a scoped work bound and
compatibility/state-migration review. A reservation does not prove key possession.
Neither it nor an authenticated cap is a total TCP or pre-fork work limit.

The independent same-source test observes legitimate admission denied after
native grace expiry, with no helper reservation charged to those rejected attempts.
Admission recovers at the sampled successful attempt around **115.4 seconds**
from the silent-cohort start, after 13 denied attempts. The listener PID,
registry inode and effective enabled penalty policy remain unchanged. Existing
members and new same-NAT callers are different availability cases: the former
survive the loaded probe; the latter can be temporarily denied. This is an
observed fairness limitation for #23/#18/#20, not a fairness pass or reason to
silently disable base protections.

CPU remains diagnostic. The active tick-based whole-host estimate and stable
owner process-runtime estimate disagree; both original derived readouts are
retained. Active raw per-role runtime sums remain in each native sample, while
the legacy interval helper stores a derived tick percentage rather than raw
endpoint tick arrays. The separate post-load control below preserves raw ticks
and clocks; it does not retroactively make the active CPU estimate a budget.

| Post-load control | Tick estimate, one-core % | Process CPU/wall % | Thread CPU/wall % |
| --- | --- | --- | --- |
| sleep | 0.000 | 0.042 | 0.042 |
| busy | 105.263 | 99.994 | 99.994 |
| periodic_30ms_busy_20ms_sleep | 51.447 | 47.946 | 47.946 |

Confidence remains limited to these observed counters and workloads. CPU
interpretation, worst-case credential/lookup mixes, longer soaks and complete
budgets stay open under #20/#28; no core-count, throughput or busy-board maximum
is adopted from these readings.

After stopping the workloads, SQLite integrity passes with 23 then 46 retained
workload posts (100,028 then 100,052 total), 1,002 unique credential hashes and
52 rooms. The bounded fixture reset then restores exactly 1,001 unique accounts,
50 rooms and 100,000 posts. Original allocator/NTP/sysctl/NPF/profile settings are
restored and read back after reboot; isolated setuid bits/mounts are removed.
The ordinary path passes 31 frontend, 20 SSH, 13 worker-lifecycle and six PTY flows.
Private VM/key/password/log/disk/forward material is removed after capture.

[Complete sanitized result 038](results/038-netbsd-sustained-abuse.json) pins the project
execution sources and installed native/helper/library/collector identities, resolved research-client
module/native-file hashes, both matrices, raw CPU controls, restoration and cleanup.
All prior sources/results/snapshots remain byte-identical. The saved-evidence
checker and mutation tests reject dropped recipients/pressure, shorter intervals,
loosened defaults/guard, source drift, erased lookup/NAT limits and missing cleanup.

## Primary evidence and confidence

- Measured installed Faction and official NetBSD components: exact hashes, versions, source maps,
  sanitized whole-host/process ledgers and observed denials in result 038.
  Narrow VM integration evidence; no physical SBC or supported capacity claim.
- [Pinned NetBSD auth2-pubkey.c](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/auth2-pubkey.c)
  and [auth2.c](https://github.com/NetBSD/src/blob/36cf5108df513943b2914535a5e7c2f86fa35fd7/crypto/external/bsd/openssh/dist/auth2.c),
  retrieved 2026-10-21 with source hashes: successful unsigned offers postpone
  authentication; failure counting returns early while postponed. This reference
  revision is not a claim that the official runtime was built from that checkout.
- [OpenSSH 10.6 release notes](https://www.openssh.org/releasenotes.html), retrieved
  2026-10-21: separately account PK_OK tests, with `PubkeyOptions max-pk-ok:nnnn` and
  a documented default of six before charging further attempts to MaxAuthTries.
  Relevant alternative for #66; not installed or validated by this experiment.
- [Paramiko Transport API](https://docs.paramiko.org/en/stable/api/transport.html),
  retrieved 2026-10-21, plus the recorded installed 4.0.0 client: KEX/host-key checks
  use the library; the synthetic unsigned auth handler is experiment-specific.

Confidence is high for recorded bounded responses/counters on this fixture;
unknown for arbitrary flood rates, worst-case sustained credentials, large
keyrings, many sources/NAT fairness, long-term leaks, production clock precision
and hostile same-UID code. Preserve this distinction when updating current docs.

[NetBSD sysctl(7)](https://man.netbsd.org/sysctl.7), retrieved 2026-10-21,
documents `kern.cp_time` as CPU-state clock ticks and aggregate output when only
one CPUSTATES array fits on multiprocessor systems. This explains the collector's
intended two-core normalization; it does not reconcile the observed statistical
and process-runtime disagreement. #20 retains CPU/timekeeping interpretation.

## Next gates and reversal

No product/schema migration. Remove optional research drivers/profile selection
to reverse the experiment; restore/read back original settings before private VM
cleanup. #55 stays open; #66 bounds repeated lookups, #23 global pre-fork/source
admission and SSH maintenance, #28 wider active/long-soak/complete budgets,
#19 independent authority/process/isolation review and #20 maintained host policy.
Lua/doors still need a separate constrained launcher/UID, never larger member caps.
