# Work and delivery contracts

These local clauses were extracted from [PROJECT.md](../../PROJECT.md) without changing their
requirements. Source identity is the consumer revision containing this extraction.
Command paths and literal configuration paths remain relative to the repository root.
Selected applicability is recorded only in root `policy.toml`; linked contracts retain
their existing scope. The owner changes required policy through the existing process.

## Delivery choices

| Area | Recorded policy |
| --- | --- |
| Work authority | GitHub Issues own scope, acceptance, priority, dependencies, progress, approaches and continuation. No local execution plan, roadmap, TODO/status journal or second task system. |
| Stack and support | Rust product, primary NetBSD; existing narrow C/FFI and Python stdlib/shell tooling boundaries remain. No JS/TS anywhere, including transitive/browser/tool/CI dependencies. |
| License | BSD-3-Clause; preserve third-party terms and rights. |
| Source control | Git; coherent issue-scoped branches/commits; PRs target main and close fully completed executable issues. No force push or branch deletion. |
| Verification | Strict behavioral TDD; explicit source/link/policy/example validation for prose. Never fabricate execution or weaken assertions/limits to pass checks. |
| Review and integration | Review the final change, resolve findings/conversations and pass current required checks against current main. Agents may merge accepted work under this workflow; a second maintainer's account approval is not mandated. |
| Releases and operations | No supported release or established release process exists. Explicit owner authorization is required for tags/releases, deployment, public listeners, host/security changes or new external services. Adoption grants none of these. |
| Documentation | Keep member/sysop docs in docs/ and technical contracts/research/evidence in dev-docs/. Use the pinned reader-oriented documentation guidance and original STE-informed prose for changed instructions; no formal ASD-STE100 compliance claim. |
| Decisions | The owner changes product direction, required policy and consequential authority. Agents resolve routine implementation choices within accepted scope and these constraints. |

Priorities/dependencies stay in GitHub. Select a finite ready issue in accepted
scope, respect ongoing ownership and ask about material priority conflicts. An
epic or deferred issue is not an executable assignment.

## GitHub state mapping

These equivalent names preserve existing labels; they do not migrate issue state.
Use one progress label on an open issue and none on a closed issue.

| Upstream state | Faction label |
| --- | --- |
| Backlog | status:needs-triage |
| Ready | status:ready |
| In progress | status:in-progress |
| Review | status:review |
| Blocked | status:blocked |
| Deferred | status:deferred |

GitHub open/closed state is completion authority. Keep priority:p0/p1/p2, area/type
labels and type:epic semantics. The [workflow](../../dev-docs/workflow.md) owns exact
readiness, PR fields, closing syntax and parent reconciliation.
