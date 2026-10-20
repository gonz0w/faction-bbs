# Stack Map

## Current state

This is a documentation-only seed repository. Its only project content is two Markdown design inputs in `docs/`; no implementation, build manifest, dependency lockfile, runtime configuration, test suite, or VCS repository is present. Consequently, no implemented technology stack can be mapped.

## Declared technology constraints (not selections)

`docs/initial-seed-prd.md` proposes, for later research:

- a backend with strong real-time support, with Elixir, Go, and Node offered as examples;
- PostgreSQL or an equivalent database with search capabilities;
- a reactive web frontend;
- REST and WebSocket APIs, plugins, and scriptable shells;
- Docker, scripts, or preferably a single-binary self-hosted deployment; and
- web, mobile, IRC, Telnet/SSH/ANSI, and other terminal-facing access options.

`docs/transport-erd.md` adds transport constraints: a transport-agnostic core, adapters for IP and Reticulum as initial research priorities, optional mesh/I2P/Tor/VPN/packet-radio support, and transport-independent local data and file storage. It mentions Reticulum libraries in Python, Go, or Rust only as possible integration material.

## Architecture-research implications

- These are product/design constraints, not adopted technologies or dependency decisions.
- Later stack selection must demonstrate durable real-time connections, persistent history, relational search, self-hosted operation, terminal compatibility, and adapter isolation.
- Transport adapters must not dictate core domain storage or business-logic types.

## Evidence

- `docs/initial-seed-prd.md` lines 44-94
- `docs/transport-erd.md` lines 21-39, 40-81, 100-107
