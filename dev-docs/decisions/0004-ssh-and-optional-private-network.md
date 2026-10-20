# 0004: SSH terminal access and optional private-network operation

Status: accepted product direction; backend/security implementation unvalidated
Date: 2026-10-19. Owner: founder, explicit answer “Yes, support that direction” in project conversation.
Issues: [#23](https://github.com/gonz0w/faction-bbs/issues/23), [#18](https://github.com/gonz0w/faction-bbs/issues/18), #6/#21/#30.
Supersedes: no chosen backend; resolves the initial secure-access direction.
Research: [appliance and access](../research/0018-appliance-and-access-direction.md).

## Decision and scope

Support ordinary SSH terminal clients as the initial secure terminal access direction.
Provide an **optional invitation-only private-network deployment mode** in which the
BBS endpoint is reachable through the selected private network. The core must also
operate without that overlay; no managed provider, public listing or HTTP is required.
No member host shell/forwarding access is implied.

Private-network admission, SSH authentication and Faction's local account/sysop
approval remain separate. Approved network/device keys never grant community access.
Preserve registration/pending handling and approved identity-art/welcome/menu flow.
Exact bootstrap/authentication mapping, SSH backend and exposure defaults remain open.

## Alternatives and rationale

Mandatory overlay adds member onboarding and external/client obligations for every
board; optional mode lets private sysops control exposure. Public SSH-only operation
would omit that requested flexibility. Plain Telnet is not the initial secure baseline;
any legacy compatibility is separately constrained in #23. No tunnels or servers were
configured by this decision.

## Gates and required validation

G1/G9 dependency/provenance audit pending; G2/G5 authentication, approval and privacy
tests pending; G3/G4/G6 local/independent/domain-separated boundaries required; G7 real
terminal/ANSI behavior pending; G8 isolated optional doors remains a separate boundary.
These are product requirements, not claimed runtime gate passes. A mandatory central
overlay violates G4; treating SSH/device identity as local approval violates G2.

Compare restricted OpenSSH gateway with embedded SSH, including complete connection
memory/CPU, host identity, registration/pending/revoked sessions, PTY/resize, deny
shell/exec/SFTP/forwarding escape paths and verify actual private-only listener/firewall
behavior. [OpenSSH primary manual](https://man.openbsd.org/sshd_config), retrieved
2026-10-19, documents forced-command and forwarding controls; configuration text alone
does not prove confinement. No SSH enforcement/protocol compatibility test ran here.

## Confidence and reversal

High confidence in explicit owner intent; implementation feasibility/resource cost
unknown. Low reversal cost now; moderate after member onboarding/keys/client guidance.
Reopen if required terminal behavior cannot be secured within footprint targets or
private-mode setup cannot be made practical. No specific SSH package/provider accepted.
