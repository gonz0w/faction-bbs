# Product Requirements Document (PRD): Self-Hosted Hybrid Community Platform

**Version**: 1.0 (Draft)  
**Date**: July 15, 2026  
**Status**: Initial Draft for Research & Iteration  

## 1. Overview & Vision

**Vision**: Create a modern, self-hosted platform that combines the best aspects of classic BBS systems (message boards, file areas, multi-user interaction, doors/games), Reddit-style communities (threaded discussions, voting, discovery), and IRC (real-time persistent chat channels).  

The result is a cohesive web-first (with terminal options) community hub that emphasizes **local-first ownership**, thoughtful interaction, and a human-scale experience.

**Core Philosophy (Anti-Algorithm Focus)**:
- Prioritize chronological or user-curated content by default.
- Minimize addictive engagement mechanics; favor quality discussions.
- Transparent, controllable recommendations (opt-in, rule-based, or agent-driven).
- Encourage deliberate participation with rate limits and community norms.
- Full data ownership with easy export and self-hosting as the primary deployment model.

## 2. Goals & Objectives

- Provide a nostalgic yet modern BBS-like environment runnable on personal homelab hardware.
- Blend asynchronous forums with synchronous chat seamlessly.
- Support private and public communities with strong moderation tools.
- Enable extensibility for custom integrations (e.g., AI agents, games, workflows).
- Lower barriers for non-technical users while satisfying power users who enjoy terminal/retro modes.

## 3. Target Users & Use Cases

**Primary Users**:
- Self-hosters and sysops managing small-to-medium communities.
- Tech enthusiasts, hobby groups, open-source projects, retro computing fans.
- Local or interest-based groups seeking private, independent spaces.

**Key Use Cases**:
- Community discussion hubs with both long-form threads and live chat.
- File sharing and resource libraries.
- Event planning, polls, and collaborative tools.
- Revival of "door" style interactive modules (games, utilities).
- Agent-assisted moderation, summarization, and workflows.

## 4. Core Features

### MVP (Phase 1)
- **Communities / Nodes**:
  - User-created topic spaces.
  - Threaded discussion boards with voting, replies, and search.
  - Integrated real-time chat channels per community (persistent history, multi-device support).

- **Content & Discovery**:
  - Anti-algorithm feeds: Chronological primary view.
  - Optional curated "best of" or followed-content sections.
  - Unified search across posts and chat.

- **Chat System**:
  - Persistent, always-on channels with notifications and history.
  - Support for web, mobile, and traditional IRC clients.
  - File sharing and basic rich interactions.

- **Classic BBS Elements**:
  - File upload/download areas with previews and organization.
  - Web interface + optional Telnet/SSH/ANSI terminal mode.
  - Extensible "doors" framework for games, tools, or embedded apps.

- **User & Moderation**:
  - Account system with roles (admin, moderator, user).
  - Basic reputation/karma system tied to community guidelines.
  - Private messaging and notifications.

- **Self-Hosting & Operations**:
  - Easy deployment (Docker, scripts, single binary preferred).
  - Backup/restore and data export.
  - Import tools from existing platforms (e.g., forums, IRC logs).

### Future Phases (Phase 2+)
- Federation for optional inter-instance communication.
- Advanced theming and retro aesthetics (ANSI, phosphor, customizable).
- Plugin/extension system for third-party integrations.
- Events calendar, classifieds, and polls.
- Deeper AI/agent integrations for automation and enhancement.
- Mobile PWA or native app support.

## 5. Non-Functional Requirements

- **Performance**: Support dozens to hundreds of concurrent users per instance depending on hardware.
- **Security & Privacy**: Strong authentication, data encryption where applicable, minimal logging/telemetry (opt-in only), GDPR-friendly export.
- **Usability**: Responsive web UI with modern and retro themes. Terminal compatibility for nostalgia.
- **Reliability**: Persistent connections, graceful degradation, easy recovery.
- **Extensibility**: REST/WebSocket APIs, plugin architecture, scriptable shells.
- **Tech Stack Guidelines**:
  - Backend: Language/framework strong in real-time (e.g., Elixir, Go, Node).
  - Database: PostgreSQL or equivalent with search capabilities.
  - Frontend: Reactive web framework.
  - Leverage existing components where possible (chat servers, forum engines, BBS backends).

## 6. Success Criteria

- Successful self-host deployment in under 30-60 minutes for basic setup.
- Positive user feedback on the balance of modern convenience and classic vibe.
- Active communities demonstrating quality discussions over high-volume noise.
- Sustainable open-source project with community contributions.

## 7. Risks & Dependencies

**Risks**:
- Complexity of unifying forum + chat data models.
- Maintaining terminal/ANSI compatibility across modern web tech.
- Moderation and spam at scale.
- Scope creep during unification.

**Dependencies**:
- Existing open-source projects (forum software, IRC/web chat clients, BBS implementations).
- Homelab-friendly runtime environments.

## 8. Research & Next Steps

- Evaluate base platforms for reuse or forking (e.g., Discourse + chat, modern BBS software, Reddit clones, Matrix/IRC bridges).
- Competitor analysis and gap identification.
- Technical spikes: Unified data model, federation protocols, terminal integration, deployment ease.
- Create user stories, wireframes, and architecture diagrams.
- Prioritize MVP features and build a proof-of-concept for one community node.

---

*This document is a living template. Expand sections with detailed user stories, acceptance criteria, technical architecture, or prioritization matrices as research progresses.*
