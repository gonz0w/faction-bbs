# Verification and execution contracts

These local clauses were extracted from [PROJECT.md](../../PROJECT.md) without changing their
requirements. Source identity is the consumer revision containing this extraction.
Command paths and literal configuration paths remain relative to the repository root.
Selected applicability is recorded only in root `policy.toml`; linked contracts retain
their existing scope. The owner changes required policy through the existing process.

## Verification and extension maintenance

Run `python3 tools/check_repository.py` and relevant tests. For this policy/checker
contract, use `python3 -m unittest discover -s tools -p test_issue_workflow.py` and
`python3 -m unittest discover -s tools -p test_check_repository.py`. Required remote
contexts are **Documentation and policy** and **Issue closure and TDD**. Their
current commands live in the [repository workflow](../../.github/workflows/documentation.yml)
and [issue workflow](../../.github/workflows/issue-policy.yml); do not copy rolling test
counts or waive failures. UI/transport/storage changes need appropriate real
PTY/native/recovery checks. Prose adoption does not require or authorize a new VM.

From `.agent-temper`, run `shasum -a 256 -c SHA256SUMS` (or the equivalent
`sha256sum -c SHA256SUMS`) and review the complete file list for unexpected additions.
Validate offline relative links and a committed source archive when provenance or
layout changes. The consumer does not need Agent Temper's Python runner.

These local extensions are versioned with Faction, outside the upstream bundle.
Their purpose is to preserve Faction's delivery, product, authority and evidence
needs. They depend on the selected profiles; verify changed contracts with their
relevant tests, current gates and separate review. The owner explicitly authorizes
replacement/removal of required policy. Routine clarifications follow bounded
issues and normal reviewed integration.

Use the [extension decision rules](../../.agent-temper/docs/reference/extensions.md) when
adoption exposes a useful practice or conflict. File a bounded upstream issue for
a general improvement or reusable profile; keep product-specific rules local.
Replace obsolete instructions deliberately and preserve history. Upstream acceptance
never updates this pin automatically. A framework update needs its own Faction
issue, source/manifest review and compatibility validation before integration.
