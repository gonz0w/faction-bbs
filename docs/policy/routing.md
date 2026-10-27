# Codex routing and resources

These local clauses were extracted from [PROJECT.md](../../PROJECT.md) without changing their
requirements. Source identity is the consumer revision containing this extraction.
Command paths and literal configuration paths remain relative to the repository root.
Selected applicability is recorded only in root `policy.toml`; linked contracts retain
their existing scope. The owner changes required policy through the existing process.

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

## Terra product routing update — 2026-10-27

The owner authorized this workflow/model-routing update and integration after
review in [the owning issue](https://github.com/gonz0w/faction-bbs/issues/236). Select `temper_product` with
`gpt-6.1-terra` / `medium` from the recorded upstream pin. This adds product-facing
work to the selected routes; the existing workflow, implementer, analyst and
reviewer settings remain unchanged. The local role is
[temper_product](../../.codex/agents/temper_product.toml).

Use the product role only for accepted documentation, visual inspection, explicitly
authorized bounded browser tasks and small UI fixes with specified behavior and
observable acceptance. Substantial implementation stays with the implementer;
ambiguous design goes to the analyst. Existing risk, platform, execution and
separate-review requirements override a routine product assignment. The author
cannot supply its own required separate review.

For this role only, the lead may explicitly select `temper_implementer` with
`gpt-6.1-sol` / `high` as the one approved fallback when Terra is unavailable or
cannot perform the task with available tools. Record that selection and confirm
capability, tool availability, existing permissions and task/resource limits
before dispatch. Missing tools are not installed by choosing another model.
Retain stricter dispatch/risk rules and stop if the fallback is unsuitable. Other
routes keep their existing fallback policy; no automatic substitution, recursive
delegation, extra retry or higher limit follows.

Terra requires a supporting client/account; Codex CLI requires 0.159.0 or newer.
Start a fresh consumer session and inspect actual role/model/effort metadata
before claiming observed routing. Configuration and source review do not prove
runtime discovery, model execution or measured quality/cost benefit. These remain
unverified until observed. This selection changes no primary-chat model, CI,
hooks, trust, permissions, product/platform or consequential-action authority.
