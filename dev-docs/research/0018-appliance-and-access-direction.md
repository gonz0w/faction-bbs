# 0018: Hobby appliance and private terminal access direction

Status: owner clarified direction; OS/SSH backend and images remain research.
Date: 2026-10-19. Related [#18](https://github.com/gonz0w/faction-bbs/issues/18), [#23](https://github.com/gonz0w/faction-bbs/issues/23), #4/#9.
Owner: founder confirms product intent; implementation recommendations require validation.
Stopping condition: resolve whether MINIX is mandatory, explain private access, and bound the next OS/access comparison. [Framework](decision-framework.md).

## Owner decisions

- Newer or older technology is welcome; measure suitability rather than age.
- Target an approachable, fast, scalable hobby BBS on small SBCs (including Raspberry Pi) or a small VM image. MINIX was an option to consider, **not a mandatory supported OS**. No specific Linux/BSD image was chosen.
- Carry forward **ordinary SSH terminal access with optional invitation-only private-network mode**, preserving independent local BBS login/registration and sysop approval. No host shell/forwarding privilege is implied for members. [Accepted direction, ADR 0004](../decisions/0004-ssh-and-optional-private-network.md).

## What private-network mode looks like

```text
member device -> optional private tunnel -> restricted SSH terminal endpoint
              -> Faction login/registration -> local sysop approval
              -> BBS identity art, welcome/menu and community features
```

The sysop authorizes a device/network key, the member enables a tunnel client, and then uses their ordinary terminal client. In private-only mode the SSH BBS endpoint binds/is reachable only through the selected private interface; firewall/default-deny behavior must be tested. Network admission, SSH authentication and BBS account/approval are distinct decisions. Revocation at any layer must have defined behavior. A private-network peer must not gain visibility into other members/host services just because it can reach the BBS.

WireGuard is a possible minimal tunnel: [official overview](https://www.wireguard.com/) and [quickstart](https://www.wireguard.com/quickstart/) describe keyed encrypted peer networking. Managed overlays can add coordination/NAT traversal/relays; [Tailscale connection documentation](https://tailscale.com/docs/reference/connection-types) describes those paths. This does not select Tailscale or a central identity provider; no service/client/network configuration was created. Compare self-managed tunnel administration, metadata/coordination, dependency footprint and service-outage behavior before a provider choice. The BBS must also operate without an overlay.

For SSH, compare dedicated restricted OpenSSH gateway against an embedded SSH implementation at equivalent policy/resource limits. [OpenSSH configuration manual](https://man.openbsd.org/sshd_config), `ForceCommand`/`DisableForwarding`, documents that forced commands alone do not disable forwarding. Do not assume each BBS member must be an OS account, or silently equate transport authentication to local approval. Prototype registration/bootstrap, PTY resize/encoding, shell/exec/SFTP/agent/TCP/socket-forwarding denial, revoked sessions, host identity, handshake/KDF resource caps and menu return before claiming a secure backend. The transport direction is accepted; none of these tests ran here.

## Whole-system OS shortlist

All primary sources retrieved 2026-10-19. This is an options screen, no measured OS ranking or supported image. RAM/image minima on a website do not equal a secured, persistent, 250-member BBS.

| Option | Primary documented fit | Tradeoff / next observation |
| --- | --- | --- |
| Minimal Alpine Linux | [Project description](https://www.alpinelinux.org/about/) describes musl/BusyBox and split packages. The [project Pi guide](https://wiki.alpinelinux.org/wiki/Raspberry_Pi) distinguishes diskless/system-disk use; page retrieval was intermittent, so recheck exact board/version instructions. | Practical first packaged-image experiment with maintained packages. Measure required kernel/modules/firmware/SSH/DB/update space; no desktop assumed. Diskless root must not make posts/files volatile. |
| Buildroot-generated Linux appliance | [Official manual](https://buildroot.org/downloads/manual/manual.html) describes cross-building toolchain, kernel, bootloader/root filesystem. | Strong control over included components, but image building/security updates/recovery fall on us. Compare build/update complexity with package-based baseline; no image built. |
| NetBSD | [Official Pi port guide](https://wiki.netbsd.org/ports/evbarm/raspberry_pi/) documents specific supported boards and differing hardware support. [Rust target list](https://doc.rust-lang.org/rustc/platform-support.html) distinguishes target guarantees. | Worth an independent compact BSD experiment; do not infer Pi ARM toolchain/SSH/DB compatibility from amd64 support or claim smaller/faster without matched builds. Driver/network/storage and update evidence matters. |
| MINIX 3 VM | [Project FAQ](https://wiki.minix3.org/doku.php?id=faq) and [QEMU guide](https://wiki.minix3.org/doku.php?id=usersguide:runningonqemu) document small-system/VM direction; listed ARM boards are BeagleBoard/BeagleBone, not an established Pi path. | Separate exploratory VM boot/toolchain/entropy/SSH/storage compatibility experiment. Rust's current target list has no named MINIX target; portability effort is unknown. Do not turn the hobby BBS into an OS port prerequisite. |
| Other trimmed Linux/BSD baseline | Same application/data/interface boundaries can be tested on another maintained host. | Keep normal development/diagnostic tools outside the runtime-image footprint, while counting the actual kernel/helpers/cache/firmware. No family is rejected by full install-media size. |

## Falsifiable comparison and gates

Next #18 experiment: same BBS revision/security policy/data/workload on a **named Pi model** and one/two-vCPU 1-GiB VM. Compare compressed download, expanded image plus update/rollback reserve, kernel/firmware/userland/application memory, idle CPU, boot-to-ready, power interruption/restore, network/storage drivers and five-step hobby installation/update/recovery effort. Separate build-host/VMM overhead from guest costs but report both. No model/image benchmark, claim of MINIX maintenance adequacy or Pi compatibility was executed here. Select exact maintained OS/build revisions before testing; no need to require all options equally for first release.

G1/G9 need full image/package/provenance audit; G2/G5 SSH/approval/privacy enforcement unknown; G3/G4/G6 are mandatory local/independent/adapter design constraints; G7 real SSH/ANSI session unknown; G8 optional doors/isolation unknown. Mandatory managed overlay fails G4, and shell/forwarding exposure or device-based approval substitution fails G2. Others remain unknown until measured. No weighted total is justified; deployment simplicity/security/complete resource costs dominate comparison. Owner intent is high confidence; runtime feasibility/OS ranking low.

Recommendation: validate one small package-based appliance and a NetBSD comparison, with Buildroot/MINIX separately bounded if needed. Keep OS adaptation outside community logic. OS/image reversal is low before deployments, moderate with sysop tooling/drivers, high if upgrades/data recovery become tied to one boot image. Accepted access intent is recorded; actual OS/backend/provider selection awaits the named tests, not another broad survey.
