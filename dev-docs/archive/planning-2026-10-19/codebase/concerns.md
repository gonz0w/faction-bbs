# Concerns and Research Constraints

## Seed Limitation

This is a documentation-only repository without code or VCS metadata. Runtime, security, dependency, schema, performance, deployment, and quality claims cannot yet be verified.

## Product Risks

Combining forums, persistent chat, files, terminal modes, doors, search, moderation, and operations presents significant scope pressure. The product's local-first and anti-algorithm principles constrain telemetry, discovery, export, and recommendation choices.

## Architecture Risks

Transport families differ in reachability, identity, discovery, latency, bandwidth, and reliability. Hybrid operation needs explicit rules for identity bindings, ordering, deduplication, persistence, retries, failures, and security. I2P/Tor, mesh, packet radio, and IP are not interchangeable initial targets.

The research plan must avoid bundling every future transport into the first prototype and must justify any scripting, plugin, federation, mothership, or mesh commitment against a concrete use case.
