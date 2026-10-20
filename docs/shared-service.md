# Shared terminal backend

The maintained Rust `faction-service` runs the full approved-member session:
board identity, sysop welcome, ANSI menus, Citadel-style rooms, local chat and
role-checked sysop controls. It shares core/terminal libraries with the local
console. The NetBSD rc.d path selects it by default; services remain opt-in,
with eight sessions and one replenished admission token per five seconds.

Use [the lifecycle/setup guide](service-lifecycle.md) and
[restricted SSH guide](restricted-ssh.md). The NetBSD native `faction-tty`
frontend holds only terminal/control descriptors; it does not link SQLite,
password hashing, owner commands or a shell parser. Base OpenSSH supplies the
transport. The server runs as a separate non-root board owner, with connection-
bound sessions and current core approval/epoch/grant checks. It is trusted with
local state; this is not a sandbox for hostile code.

Themes load once at startup. [Sysop packs](theme-packs.md) support safe CP437
identity, arrow/Enter and displayed-hotkey navigation, manual seasons, plain
fallback and resize recovery. Rooms support selected pages, unread cursors,
replies and exact post retries. Chat preserves unsent drafts across arrivals;
permission changes discard the draft and require an explicit fresh join.

Input, encoded output, background work and credential dispatch are bounded. Two
fixed credential threads share the unchanged two-slot cross-process Argon2 budget
with owner tools. There are no per-member application workers in shared mode;
each SSH session still has mandatory frontend and OpenSSH costs. Service failure
affects all attached members. The worker backend remains a tested rollback choice
using the same database schema.

[The original candidate](shared-ui.md) and historical research remain preserved.
The current service integration advances #55; broader active mixes/soak/admission,
stronger confinement and maintained image/timekeeping policy remain separate
#28/#20 work. Explicit experimental 250-session configurations and short VM
measurements do not establish supported capacity or physical SBC behavior.

Lua menus and isolated doors remain the next extension priority after the useful
core. Files, mail, federation, browser access and external networks remain backlog
items; the shared service does not enable them implicitly.

[Hardened installed-service evidence](../dev-docs/research/0034-process-confinement.md)
records three fresh-boot 250-member runs with 137.61 MiB lowest sampled stress
headroom. The eight-session default is unchanged; the larger research profile
is not a supported capacity setting.

See [native process confinement](process-confinement.md) for the current peer-UID,
TTY/descriptor/environment checks, irreversible resource ceilings and remaining
filesystem/network/exec isolation boundary.
