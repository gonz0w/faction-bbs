# Repository Structure

## Present layout

```text
docs/
  initial-seed-prd.md       Draft product requirements
  transport-erd.md          Draft transport-agnostic design and high-level ERD
```

This is the complete seed layout observed during mapping. There are no implementation directories, product source files, package manifests, build scripts, tests, deployment files, generated assets, or VCS metadata.

## Document roles

- `docs/initial-seed-prd.md` establishes the vision, MVP/future feature scope, non-functional requirements, risks, dependencies, and proposed research next steps.
- `docs/transport-erd.md` constrains future architecture around a transport-agnostic core, adapters, clients, storage, candidate networks, and a high-level data-model outline.

## Implication for future layout work

Any future application layout is undecided. A later architecture phase should separate core domain logic, transport adapters, client/terminal surfaces, persistence, file storage, and operational tooling—but that is a research direction stated in the inputs, not an existing repository structure or adopted design.

## Evidence

- repository root inspection: only `docs/` present
- `docs/initial-seed-prd.md` lines 1-125
- `docs/transport-erd.md` lines 1-136
