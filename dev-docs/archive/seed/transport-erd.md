# System Design Document: Multi-Network & Transport-Agnostic Architecture

**Version**: 1.0 (Draft)  
**Date**: July 15, 2026  
**Status**: Initial Draft for Research & Planning  

## 1. Overview & Goals

**Vision**: Design the hybrid BBS/forum/chat platform with a **pluggable, transport-agnostic architecture** so it can run seamlessly over traditional IP-based networks (ISPs) as well as alternative networks such as Reticulum, meshes, I2P/Tor, local ad-hoc, packet radio, and future options.

The system should be **easy to configure** for different transports with minimal changes to core features (forums, chat, file areas, doors). Prioritize modularity so administrators can enable one or more transports simultaneously.

**Key Objectives**:
- Maximize compatibility with diverse networking stacks for resilience, privacy, and local-first use.
- Provide simple setup (config files, Docker profiles, or wizards) for common transports.
- Maintain consistent user experience (web UI, terminal mode) regardless of underlying transport.
- Support hybrid operation (e.g., IP + Reticulum simultaneously).

**Anti-Algorithm & Sovereignty Alignment**: Enable truly independent, censorship-resistant communities by reducing reliance on any single network provider.

## 2. High-Level Architecture

- **Core Application Layer** (Transport-Agnostic):
  - Business logic for users, posts, chat messages, files, reputation, moderation.
  - Unified data model (e.g., shared database for threads, messages, users).

- **Transport Abstraction Layer**:
  - Interface/adapter pattern for different networking backends.
  - Handles addressing, discovery, message routing, and persistence.

- **Frontend / Client Layer**:
  - Web UI (served locally or via gateway).
  - Terminal/ANSI mode.
  - Native clients or bridges for specific networks.

- **Storage & Services**:
  - Local database (PostgreSQL or lightweight embedded).
  - File storage independent of transport.

## 3. Supported Transports & Integration Strategy

The system will provide **first-class, easy-to-enable support** for the following (prioritized by feasibility and value):

### High Priority
- **Standard IP / ISP**:
  - TCP/UDP, WebSockets, HTTP(S).
  - Setup: Default Docker Compose profile, domain/SSL config.

- **Reticulum**:
  - Native integration via Reticulum Python/Go/Rust libraries.
  - Leverage existing RetiBBS and RRC patterns for BBS + chat.
  - Setup: Enable Reticulum interface in config; automatic identity management.

- **Mesh Networks** (e.g., Meshtastic, BATMAN, local WiFi/LoRa):
  - Peer-to-peer propagation with store-and-forward.
  - Setup: Mesh adapter module with discovery.

### Medium Priority
- **I2P / Tor Hidden Services**:
  - Anonymous hosting.
  - Setup: One-click hidden service generation (similar to self-hosted tools).

- **VPN / Overlay Networks** (Tailscale, WireGuard, ZeroTier):
  - Easy "private internet" mode.
  - Setup: Auto-detect or simple config for VPN interfaces.

- **Packet Radio / AX.25 / Ham**:
  - Classic BBS compatibility.
  - Setup: Serial/TNC integration module.

### Future / Extensible
- Delay-Tolerant Networking (DTN).
- Bluetooth/NFC sneakernet bridges.
- Custom or emerging stacks via plugin system.

**Ease of Setup Principles**:
- Single configuration file or UI wizard for enabling transports.
- Docker Compose profiles (e.g., `docker compose --profile reticulum up`).
- Automatic detection of available interfaces.
- Clear documentation and example configs for each.
- Fallback to IP if alternatives are unavailable.

## 4. Data Model Considerations (High-Level ERD Outline)

**Core Entities** (Independent of Transport):
- **User** (identity across networks, with transport-specific addresses).
- **Community/Node** (forum + chat space).
- **Post/Thread** (unified for async discussions).
- **ChatMessage** (real-time or store-and-forward).
- **File** (attachments, shared library).
- **Identity/Address** (per-transport, e.g., IP:port, Reticulum destination hash).

**Relationships**:
- User belongs to multiple Communities.
- Communities contain Posts and Chat channels.
- Messages/Posts can be tagged with transport metadata (for hybrid routing).

*(In implementation: Use a relational DB with optional NoSQL for messages. Transport layer maps addresses without polluting core models.)*

## 5. Technical Implementation Guidelines

- **Adapter Pattern**: Each transport implements a common interface (`send_message`, `subscribe`, `discover`, `connect`).
- **Libraries**: Reticulum SDK, Tor/I2P clients, standard socket libs.
- **Discovery**: Unified user/community discovery that works across enabled transports.
- **Security**: E2EE where possible; transport-specific auth (e.g., Reticulum identities).
- **Performance**: Graceful degradation on constrained networks (e.g., compress messages, prioritize async BBS over live chat).
- **Testing**: Unit tests per adapter + integration tests for hybrid scenarios.

## 6. User & Admin Experience

- **Admin Setup Flow**:
  1. Install core system.
  2. Choose/enable transports in config or setup wizard.
  3. Generate identities/keys as needed.
  4. Launch and share access instructions per transport.

- **User Experience**:
  - Connect via preferred method (web link, Reticulum client, terminal).
  - Seamless experience inside communities regardless of how they connected.

## 7. Risks & Mitigations

- **Complexity**: Keep core simple; isolate transport code.
- **Maintenance**: Start with 2-3 transports (IP + Reticulum) and expand.
- **Compatibility**: Provide gateways/bridges for non-native clients.

## 8. Next Steps

- Research and prototype adapters for IP and Reticulum.
- Define detailed interface contracts.
- Evaluate existing projects (RetiBBS, RRC, Discourse/Matrix bridges) for reuse.
- Create PoC for hybrid operation.

---

*This document complements the earlier PRD. It focuses on the multi-network design while keeping setup approachable. Expand with diagrams, code skeletons, or detailed adapter specs as needed.*
