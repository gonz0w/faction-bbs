# Integration Map

## Implemented integrations

None. This documentation-only seed contains no source code, configuration, credentials, service clients, package manifests, or deployment definitions. No integration is installed, configured, or verified.

## Future-facing integration constraints

The PRD identifies possible integration categories: traditional IRC clients, imported forum and IRC-log data, AI agents, games/doors, third-party plugins, REST/WebSocket consumers, and backup/export tooling. These are requirements for later research rather than commitments.

The transport design asks for a common adapter interface with operations such as `send_message`, `subscribe`, `discover`, and `connect`; it identifies IP/HTTP(S)/WebSocket, Reticulum, mesh networks, I2P/Tor, VPN/overlay networks, and packet radio as candidate transport families. A consistent client experience and simultaneous multi-transport operation are stated goals.

## Boundary constraints for later design

- Keep transport addressing, discovery, routing, and connection behavior behind adapters.
- Keep core users, communities, posts, chat, files, reputation, and moderation independent of individual transports.
- Treat import/export, client bridges, plugin APIs, and agent integrations as separately permissioned and failure-isolated boundaries.
- Do not infer credentials, network endpoints, APIs, or providers from these documents.

## Evidence

- `docs/initial-seed-prd.md` lines 35-40, 55-63, 70-80, 83-94, 111-121
- `docs/transport-erd.md` lines 23-39, 42-81, 100-107, 111-125
