# Repository Structure

## Present Layout

```text
docs/
  initial-seed-prd.md       Draft product requirements
  transport-erd.md          Draft transport-agnostic design input
```

This is a documentation-only seed. There are no implementation directories, package manifests, build scripts, tests, deployment files, generated assets, or VCS metadata.

## Document Roles

- `docs/initial-seed-prd.md` describes product vision, feature scope, non-functional requirements, risks, and research next steps.
- `docs/transport-erd.md` describes transport-agnostic constraints, candidate networks, and a high-level data-model direction.

Any application layout remains undecided. Future research must distinguish core domain logic, transports, clients, persistence, extension points, and operations without treating that separation as an adopted design.
