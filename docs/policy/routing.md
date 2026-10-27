# Product routing extension

These local clauses were extracted from [PROJECT.md](../../PROJECT.md) without changing their
requirements. Source identity is the consumer revision containing this extraction.
Command paths and literal configuration paths remain relative to the repository root.
Selected applicability is recorded only in root `policy.toml`; linked contracts retain
their existing scope. The owner changes required policy through the existing process.

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
