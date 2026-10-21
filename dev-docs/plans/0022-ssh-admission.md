# 0022: Bounded SSH admission and abuse costs

Issue: [#55](https://github.com/gonz0w/faction-bbs/issues/55), related #19/#20/#28.
Status: implementation/native/host validation complete; focused PR and CI pending. #64 merged at `0c668ec`.

## Goal and constraints

Bound authenticated reservations using base SSH key authorization, while retaining
base preauthentication controls, without weakening base OpenSSH, BBS approval, fixed credentials/resource limits,
eight-session defaults or the unchanged 128-MiB/zero-swap experiment predicate.
Preserve executed 035/036 source bytes: use new files rather than rewrite them.
No physical hardware, JS/TS, per-IP member tracking or automatic host/firewall tuning.

## Plan

- [x] Verify/merge #64 and read current issue/constraints/evidence.
- [x] Compare base inetd/OpenSSH and prototype bounded admission.
- [x] Preserve the rejected listener and live-ledger diagnostic; retain guard.
- [x] Implement base-SSH key hook, private birth registry and quiescent preflight.
- [x] Native 14 key/state/rate/config/restart, six channel/restart and 29 UI checks.
- [x] Three fresh selected 250-member full UI/credential/control/limit workloads.
- [x] Native silent-preauth, 31 sealed/31 ordinary frontend and ELF/analyzer checks.
- [x] Restore settings/reboot/readback, ordinary 20 SSH/13 lifecycle/six PTY checks,
  seed integrity/counts and private VM/key/log/disk/forward cleanup.
- [x] Final host evidence/repository checks: 52 Python tests; all prior saved
  evidence/source checks pass.
- [ ] Focused PR and required exact-head CI.

## Findings and current candidate

The first native C listener/sshd-inetd candidate completed three functional
250-member runs, but stress minima were 129.35/125.50/126.04 MiB. Two fail the
unchanged guard: reject it for this target, preserving actual sources/results.
A config-sender zombie per connection also required explicit process accounting;
the earlier 32-member live-rlimit instrumentation failure remains preserved.

The selected experiment uses base OpenSSH's AuthorizedKeysCommand under a separate
noninteractive helper UID, a persistent fixed process-birth registry, global
reservation/token caps and strict root preflight with no fallback key-file path.
No resident daemon, proxy, SSH parser or base SSH signal/crypto change. This caps
key-admission/authenticated monitors; it does not impose a global pre-fork rate
limit. Base MaxStartups/penalties/timeouts remain. Up to eight additional preauth
connections and shared-NAT penalty behavior remain explicit limits.

Native 14 key/state/restart/rate/config/missing-state checks, 29 full UI and six
SSH channel/restart checks pass with the final helper. All three selected
fresh-boot 250-member runs pass every guard (130.01/130.09/129.34 MiB minima).
Native boot initializes state only after proving quiescence. Preserve both
rejected memory controls and exact diagnostic input bytes. Normal defaults remain eight.
