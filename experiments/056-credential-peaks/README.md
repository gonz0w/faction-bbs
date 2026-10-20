# Persistent credential peak experiment (#56)

`netbsd_metrics.c` derives from the saved #54 collector without changing that
historical source. Compile natively on NetBSD with `cc -std=c11 -O2 -Wall -Wextra
-Werror`. It takes backend UID, terminal UID and dedicated SSH listener PID; these
inputs are never printed. Snapshot mode retains the prior sanitized metric format.
`--stream INTERVAL_MS COUNT` produces explicit start/sample/stop NDJSON, guest
monotonic query-start/query-finish times and the requested interval. Limits are
5–1000 ms, 12000 samples. Missed deadlines are skipped, never replayed in a burst.
The process remains alive over one SSH channel; no per-sample process/SSH startup.
The copied whole-VM counters, full dedicated SSH ancestry, per-role RSS and CPU
retain the earlier accounting limits: RSS sums are not private memory or PSS.
The shared experimental owner is classified in the broker role.

`calibrate.c` touches 0–64 MiB in page steps, holds it 20–2000 ms and emits only
byte count and monotonic allocation markers. It contains no BBS credentials.
`tools/check_credential_peaks.py` calibrates 0/19/38 MiB for 200 ms, then measures
eight distinct approved accounts already at hidden password prompts. It runs
two held-slot saturation, incorrect-password, valid-password and repeated valid
bursts, and a fresh approved login after release. It does not change or slow
Argon2. Two cross-process locks remain the concurrency bound; cumulative successes
can exceed two. The collector does not probe/acquire locks and cannot consume a
credential slot. Known external holders are explicit saturation inputs.

`tools/credential_peak.py` bounds stream bytes/events and validates complete,
ordered monotonic samples and sanitized process metrics. It reports measured
cadence/gaps, query duration and host versus guest elapsed time. Calibration uses
only samples fully inside the touched allocation interval, relative to prior
samples. A sampled maximum is a lower bound on the actual peak; it is never an
instantaneous upper bound. No inference of simultaneous KDF count from cumulative
successes or lifetime RSS sums is allowed. Kernel/cache/SSH noise remains visible.

Run only on a fresh signed/hash-verified, marked disposable VM with pinned private
keys and generated synthetic fixture data. Drivers require `--disposable-fixture`;
secret input is a private local file, never a command argument or saved transcript.
Freeze source/binary identity, sanitize evidence and delete guest/disks/keys/logs/
forwards afterwards. This is optional research tooling, not a required runtime.
