# File areas and extensible addressed mail

Owner steering 2026-10-21; #22, #33, #17, #81. Based on the original LORD proof
branch while PR #82 is open; do not silently merge #82 without authorization.
Status: design/research complete; implementation queued as #83/#84.

## Goal

Prioritize useful BBS file areas. Keep file storage/permissions independent of
terminal transfer protocols. Research a separate opt-in addressed private-mail
adapter, initially comparing XMPP reuse; preserve local-only defaults and all
membership/credential/confinement gates. The owner's new mail request supersedes
the earlier blanket chat-only federation rule only for deliberately addressed
private mail. No automatic replication or recipient-board admission.

Games pause after the current proof. Preserve future DOS/Amiga/other-host adapters
behind bounded terminal I/O; emulator availability does not make a graphical game
usable through ANSI/TTY. No native code launches inside the BBS owner/frontend/Lua.

## Work

1. Read #22/#33/#17 and current domain, snapshot, terminal and service boundaries.
2. Compare transfer protocols and XMPP/local relay alternatives using primary
   specifications. Clarify transfer experience and encryption endpoint ownership.
3. Record current requirements, decisions, limits and concrete GitHub follow-ups;
   avoid promising XMPP compatibility/E2EE or new emulator support from designs.
4. Queue the bounded file-area implementation under #83: local approval, safe
   metadata, private storage/quarantine, deterministic limits and complete backup.
   Owner's later identity clarification makes this turn a design/research slice;
   no transfer implementation is claimed here.
5. Define the meaningful authorization/path/quota/interruption/persistence checks
   for #83 and client/mail proofs for #84. Run repository checks on this doc-only
   change; preserve measured source/record bytes when code work begins.
6. Update README, AGENTS, docs/dev-docs/CHANGELOG and open a focused PR with exact
   completed behavior and remaining work. Keep original-game #82 separate.

## Owner decisions and remaining protocol work

- SFTP with a separate restricted service first; classic terminal transfers later.
- Server-readable mail rejected. Native local terminal client accepted; device keys
  and plaintext stay on member devices, independent of visited-board SSH output.
- Independent invited home servers accepted; founder's board first peer, no
  permanent mandatory central identity service. Local approval remains per board.
- Concrete SFTP admission/handler and XMPP/E2EE/visiting-board auth profiles remain
  implementation experiments. #83/#84 hold acceptance; no runtime claims or
  current BBS auth/schema changes. This turn records the product/architecture
  decisions rather than silently implementing a weaker mail model.
