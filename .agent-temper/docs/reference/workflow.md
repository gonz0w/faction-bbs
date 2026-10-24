# Workflow

This reference expands the [core method](../../METHOD.md). A work item, review,
revision, and integration record are portable concepts. A project profile maps
them to actual tools and commands.

## Discover and decide

Understand the user, problem, intended outcome, constraints, and current system.
Read the selected work item and relevant discussion before changing artifacts.
For ambiguous work, ask focused questions and continue independent investigation.
Research has a question, a bounded investigation, and a decision or observation
as its deliverable. It does not automatically authorize implementation.

Create an implementation work item before implementing. A work item can contain
its specification, approach, and validation; do not create a parallel plan file.
Make a lasting interface or architecture decision a linked document when needed.

## Set milestones and slice work

A milestone names an observable capability and its exit criteria. Group bounded
deliverables that establish that capability. Dependencies describe actual ordering
constraints. A calendar date or completed-task percentage does not prove readiness.

Plan the current increment in detail. Refine later work when its prerequisites are
known. Split by independently useful outcomes, ownership, contracts, or review
risk. Keep related positive and negative test cases within the same work item.
Do not divide work solely to meet a line, file, or ticket count.

## Select ready work

Ready work has a clear outcome, non-goals, acceptance, verification, cleared
dependencies, and authority to proceed. A work item may be ready for research
while implementation is still blocked; name the permitted outcome explicitly.

On a continuation request:

1. Read current project policy and the authoritative work record.
2. Inspect active changes and unresolved reviews before starting another change.
3. Select the highest-priority coherent ready item within the authorized scope.
4. Record ownership and the brief approach in that item.

A blocked item names the blocker and what would remove it. Deferred work is not
current authorization. If no ready work is authorized, ask the owner to choose or
clarify it; do not invent a product direction.

## Implement and coordinate

Inspect current source and relevant tests. Establish expected behavior before
changing it. Use the [verification reference](verification.md) for the change type.
Implement the smallest complete outcome and update affected documentation.

One lead owns the integrated deliverable. Delegated work has an explicit boundary,
inputs, expected output, and verification. Inspect shared edits before overwriting
files. Separate working copies or nonoverlapping ownership can reduce collisions;
the method does not require a particular worktree tool or agent topology.

Resolve routine implementation choices under the project's delegation policy.
Stop affected work if policy is missing, conflicting, or requires an owner decision.
Do not use a timeout, silence, or another agent's preference as approval.

New unrelated defects and enhancements become new work items before implementation.
A change needed to meet existing acceptance stays in the current item. If the
acceptance itself is wrong, explain the evidence and get the required authority
before changing the expected result. Do not silently move the goalposts.

## Review and integrate

Create coherent changes with messages that explain the outcome and reference the
work item. Inspect the actual changed files, exclude unrelated work and private
data, and preserve meaningful history. Branch naming, commit syntax, signatures,
and merge strategy belong to the version-control/project profile.

The review record states the problem, resulting behavior, acceptance-to-evidence
mapping, consequential decisions, and remaining limits. Review current integrated
inputs and their affected consumers. If inputs change, refresh the relevant checks.
An agent review is a useful second examination, not proof of independent assurance.

Integrate only within recorded authority and after the required checks and review.
Close implementation work only when its completion conditions are satisfied.
An unintegrated or abandoned change leaves unfinished work open. Reconcile linked
dependencies and parent outcomes; a merged component cannot establish untested
system acceptance.

## Release and operate

Use the project's explicit release and deployment authority. Identify the source
revision or revision set, built artifacts, supported targets, release notes, and
applicable qualification. Where behavior or data changes, provide compatibility,
migration, backup, rollback, and post-change verification appropriate to the risk.
Record unavailable checks and do not extend support claims beyond the evidence.

For an incident, follow the project's incident authority and containment/runbook
policy. Restore safe service, preserve useful evidence, and track corrective work.
An urgent action uses an explicitly authorized emergency path; urgency alone does
not grant new permissions. Remove secrets and personal data from public records.

Maintain dependencies and support policies through bounded updates. Deprecate or
remove interfaces only after checking users, communicated promises, and migration
needs. Use observed defects and review friction to improve the method itself.

## Leave a usable continuation

Keep a compact checkpoint in the work item when work is interrupted or handed off:

- Current source revision or working-change identity and review link.
- What has been completed, with actual verification and its scope.
- What remains, including a named blocker or unresolved decision.
- The next concrete action and any authority needed for it.

Do not paste a full conversation or maintain a second status journal. A new agent
must recheck current state; the checkpoint is evidence and orientation, not a
guarantee that source, permissions, or external systems remain unchanged.

