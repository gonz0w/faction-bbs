# Faction BBS agent instructions

Read [PROJECT.md](PROJECT.md), the [pinned Agent Temper method](.agent-temper/METHOD.md),
and the selected GitHub issue **with its comments** before editing. PROJECT.md
selects the applicable profiles and preserves Faction's stronger local contracts.
Follow the [Faction workflow](dev-docs/workflow.md) for exact issue/PR/check rules.

- **GitHub Issues are the only work authority.** No issue means no implementation;
  create a finite executable issue first. Approaches, discoveries, red/green and
  continuation belong in issue comments. No local plans, backlog or agent journal.
- Apply recorded policy without re-asking. Missing or conflicting required policy
  stops dependent work for owner Q&A. Upstream updates never silently change the
  adopted revision or local authority.
- Use strict test-first delivery: run meaningful behavior tests red before
  implementation, then green with assertions intact. Prose uses actual link/source/
  policy validation. Never invent TDD, weaken checks or tune limits to pass CI.
- Read [product direction](docs/project.md), [privacy/security](docs/security-privacy.md),
  [architecture](dev-docs/architecture.md) and relevant feature docs/ADRs. Preserve
  Rust, primary NetBSD, no JS/TS, device-private keys/mail, classic access, current
  approval/resource boundaries and deferred scope in PROJECT.md.
- Before editing measured inputs, read [evidence preservation](dev-docs/evidence/README.md)
  and preserve actual executed bytes. Never refresh old hashes/results to new code.
- Run relevant checks and required CI. Review the final change, finish each issue's
  acceptance, use a real closing line per completed issue, and reconcile closure,
  parents and progress labels after merge. Partial work needs a bounded child first.
- The owner playground stays **stopped and intact** unless the owner asks to run it.
  Use marked owned disposable fixtures and prove teardown. Never publish keys,
  private endpoints, member data or unlicensed games/art. Use SECURITY.md.
