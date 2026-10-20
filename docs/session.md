# The BBS session

Status: product flow confirmed; screens, keys, protocols, and storage are not implemented.

## Entry and membership

Users connect from a network terminal application or, where enabled and feasible under the no-JS constraint, a browser console. They log in or submit a registration request. Registration creates no automatic community access. Pending, rejected, or revoked users cannot read boards, join chat, browse files, use mail, launch doors, or open the other-BBS directory. Entry/registration/status feedback is the only pre-approval experience to design.

Approval is controlled locally by the sysop. Both authentication and approval must be enforced by the server for every community action, not just by hiding menu entries. Role and recovery details remain research questions.

## Approved login

1. Display the BBS's own ANSI identity artwork so the visitor knows where they landed.
2. Present the main menu and a sysop-defined message, broadcast, quote, or welcome content.
3. Let the user navigate the locally available areas and return to the menu.

Artwork capability negotiation, bandwidth limits, safe escape handling, skip behavior, and plain-text/accessibility alternatives need validation. The intended order preserves the BBS identity screen before the main menu.

## Candidate main menu

| Entry | Intended role |
| --- | --- |
| Message rooms | Citadel-style room discovery, catch-up, posting, replies. |
| Chat | Command-driven conversation; identify local-only and future shared channels clearly. |
| Mail | Local member messaging, subject to later requirements. |
| File areas | Sysop-controlled uploads/downloads, resources, access policies. |
| Doors / games | Optional external games with a reliable return path. |
| Contacts / Other BBSes | Local system listings and visits to another operator's board. |
| Account / preferences | Personal settings and terminal capabilities. |
| Help / contact sysop | Navigation help and local operator contact. |
| Logout | Deliberate session exit. |

This is a research inventory, not a fixed key map or a first-release feature commitment. Sysops should be able to shape relevant menus within permission and upgrade boundaries. Administrative tools require the appropriate local role.

## Between systems

An operator may submit their BBS to another board's local directory. Review policy and listing lifecycle remain open. A visit must honor the destination's own login and approval; no automatic account transfer is assumed. Local content remains on its originating BBS. Only future explicitly participating shared chat channels cross system boundaries.

## Validation targets

Demonstrate the complete approval gate and artwork/menu order on both available access surfaces, persistence of room read/post state after interruption, chat/menu transitions, door entry/return and failure isolation, and a directory visit that grants no remote access. Set measurable payload/latency and terminal-capability conditions before experimenting.
