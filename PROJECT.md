# Faction engineering policy

This lasting policy adopts Agent Temper through [issue #188](https://github.com/gonz0w/faction-bbs/issues/188)
on 2026-10-23 and preserves Faction's existing owner-approved constraints.
Live assignments and continuation remain exclusively in GitHub Issues.

## Adopted method and local authority

Read the [pinned method](.agent-temper/METHOD.md). Its exact reviewed revision is
recorded in [SOURCE.md](.agent-temper/SOURCE.md); the
[SHA-256 manifest](.agent-temper/SHA256SUMS) identifies the unchanged upstream files.
The bundle retains its [BSD-3-Clause license](.agent-temper/LICENSE).

Only these profiles are selected:

| Profile | Purpose and local composition |
| --- | --- |
| [GitHub](.agent-temper/profiles/github.md) | One issue authority and finite delivery, with the local state mapping below and exact PR/check rules in the workflow. |
| [Strict test-first](.agent-temper/profiles/strict-test-first.md) | Actual red/green/refactor for behavior, including executable tooling/configuration. Preserve Faction's stronger evidence rules and fixed historical exception. |
| [Codex](.agent-temper/profiles/codex.md) | Explicit project model routing under the local selection below; existing verification and authority remain binding. |

Other bundled profiles are not active. The product/resource contracts below,
[privacy/security](docs/security-privacy.md), accepted
[decisions](dev-docs/decisions/README.md) and [workflow](dev-docs/workflow.md) are
Faction extensions; they do not become universal Agent Temper requirements.

A compatible stronger local rule remains binding. Neither an upstream update nor
file order replaces an adopted decision. A missing or conflicting required policy
stops dependent work for owner Q&A; continue independent work. Record the answer
here or in its linked canonical contract before proceeding. Settled choices persist
across agents and sessions.

## Delivery choices

Read the canonical [work and delivery contracts](docs/policy/work.md#delivery-choices).

## GitHub state mapping

Read the canonical [work and delivery contracts](docs/policy/work.md#github-state-mapping).

## Verification and extension maintenance

Read the canonical [verification and execution contracts](docs/policy/verification.md#verification-and-extension-maintenance).

## Codex routing selection

The owner authorized this profile selection and bundle update in
[issue #193](https://github.com/gonz0w/faction-bbs/issues/193). It supplements the
GitHub and strict test-first profiles. It does not replace existing product,
privacy, resource, evidence, review, integration or operational rules. Earlier
active work keeps its accepted scope and verification; future delegation follows
this explicit selection. The previous pin remains recoverable in Git history.

| Role | Assignment | Model | Effort |
| --- | --- | --- | --- |
| `temper_workflow` | Bounded read-only inspection and evidence summaries | `gpt-6-luna` | `medium` |
| `temper_product` | Bounded documentation, visual checks, authorized browser tasks and small specified UI fixes | `gpt-6.1-terra` | `medium` |
| `temper_implementer` | Accepted implementation, tests and ordinary debugging | `gpt-6.1-sol` | `high` |
| `temper_analyst` | Difficult analysis or separately assigned complex review | `gpt-6-astra` | `high` |
| `temper_reviewer` | Separate final review of work it did not author | `gpt-6.1-sol` | `high` |

Local role files live in `.codex/agents/`, outside the immutable bundle. The lead
assigns bounded work and owns tracker writes, integration and returned-evidence
verification. Use a maximum of three concurrent delegated agents, excluding the
lead, and stay within existing resource limits; this does not enlarge the three
owner-helper runtime budget. No nested helpers without an explicit lead assignment.
Tiny or tightly coupled work may stay with the lead. No role changes permissions.

If a selected role or model is unavailable, report that result. The lead may do
the bounded work with its current model only when adequate for the task and
permitted by existing policy and resource limits; otherwise resolve the missing
choice with the owner. Never silently substitute a model or claim requested
settings were observed. Do not raise effort or change model without the lead's
assignment within this selection. The owner approves changes to this selection;
routine task routing within it is delegated to the lead.

No project global/default model or permission setting is introduced. The primary
chat keeps its selected model. In a runtime that cannot discover named roles but
supports explicit model settings, the lead may use the same table as an equivalent
mapping with a complete fresh-context brief and recorded observed settings.
Otherwise use the fallback above. A parse check proves syntax only. Record actual
client/version and role/model/effort metadata when available in the adoption
issue; mark native discovery or routing unverified until observed in a fresh
supporting session. Existing chats do not switch models when files change.

## Product and implementation contracts

Read the canonical [product and compatibility contracts](docs/policy/product.md#product-and-implementation-contracts).

## Accepted first optional roaming model

Read the canonical [product and compatibility contracts](docs/policy/product.md#accepted-first-optional-roaming-model).

## Resource and evidence contracts

Read the canonical [resource and evidence contracts](docs/policy/resources.md#resource-and-evidence-contracts).

## Terra product routing update — 2026-10-27

Read the canonical [codex routing and resources](docs/policy/routing.md#terra-product-routing-update--2026-10-27).

## One-time Terra routing integration decision

Read the canonical [historical finite decisions](docs/policy/history.md#one-time-terra-routing-integration-decision).

## One-time issue #239 oracle TDD disposition

Read the canonical [historical finite decisions](docs/policy/history.md#one-time-issue-239-oracle-tdd-disposition).

## Explicit policy selection

The owner-selected modular adoption in [issue #246](https://github.com/gonz0w/faction-bbs/issues/246)
uses root [policy.toml](policy.toml) as the sole selected-module record. Read every
selected module and its declared prerequisites before work; linked local contracts
retain their existing applicability. PROJECT remains the entry point and retains
its prior heading anchors as pointers to the canonical extracted clauses.

Compatible stronger rules apply together. The owner controls required-policy,
selection, exception and authority changes through the existing work/review process.
The manifest describes composition; it does not authorize actions or supersede prose.
Historical finite approvals remain expired for this work; selection does not renew
WIP lanes, host permissions, check/hook waivers or operational authority.

The [manually invoked selection checker](tools/temper/SOURCE.md) reads inert declared data.
It is outside the retained method bundle and is not wired into CI, hooks or gates.
Mechanical success leaves policy meaning, source trust, approval, check applicability
and execution/integration authority unverified. Use only an already authorized executor;
missing required checks remain unavailable and block their dependent acceptance.
