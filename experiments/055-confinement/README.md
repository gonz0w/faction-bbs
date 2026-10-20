# #55 native confinement oracles

Disposable marked NetBSD VM only. No sysop deployment automation or secrets are
included. Read [the process contract](../../docs/process-confinement.md) and
[research 0034](../../dev-docs/research/0034-process-confinement.md).

- `process_probe.c` links the exact maintained `process_security.c` with pthreads.
  Separate dropped-UID children validate frontend and three-helper-thread service
  limits, denied fork/extra thread/mlock/files/limit increases, plus an actual
  SIGABRT with no core file in an owner-writable isolated directory.
- `frontend_probe.py` runs the installed native frontend under the terminal UID
  with real PTYs. Positive mock sockets bind under the actual owner UID. The
  wrong-root-peer case uses correct chowned pathname metadata and verifies that
  no descriptor is sent. Thirty-one cases cover existing and added denials.
- `startup_probe.py` executes the installed shared service with unsafe inherited
  identities/groups/FDs, requiring failure before listen and unchanged database
  digest. The same configuration must start/stop via the real clean rc.d path.
- `terminal_probe.py` sends real TTY descriptors from wrong root/owner kernel
  peers, then drops to the configured terminal UID for valid control/slow-reader/
  recovery probes. Path ownership alone cannot satisfy its identity checks.

Build the process probe natively in the marked private fixture:
+
+```sh
+cc -std=c11 -O2 -Wall -Wextra -Werror -pthread \
+  experiments/055-confinement/process_probe.c \
+  crates/faction-unix/src/process_security.c \
+  -o /usr/local/libexec/faction-confinement-probe
+/usr/local/libexec/faction-confinement-probe
+```
+
+Run native workspace tests/build and frontend build first. Install the opt-in
root-managed service/SSH fixture, seed synthetic data, and keep generated secrets
private. The host `tools/check_integrated_oracles.py` invokes the direct terminal
probe alongside the full UI campaign; lifecycle checks require the dedicated SSH
listener to be running. Worker regression teardown stops that listener, so restart
it with its backend dependency before switching campaigns. A temporary diagnostic
frontend was restored before counted tests; it is not a measured final binary.

Use the existing versioned service scaling/host drivers for all three fresh-boot
250-member trials. Keep complete samples, control windows, native live limits and
SSH roles; never replace native observations with this documentation or a synthetic
model. `tools/check_confinement_evidence.py` checks the saved record, current hashes
and every owner/frontend ceiling; it does not rerun performance measurements.
