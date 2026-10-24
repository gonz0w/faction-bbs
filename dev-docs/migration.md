# Retiring repository task tracking

The founder retired `.planning/` on 2026-10-19 and explicitly required 100% GitHub
issue-driven work under [#181](https://github.com/gonz0w/faction-bbs/issues/181) on
2026-10-24. GitHub Issues now exclusively hold backlog, acceptance, dependencies,
execution approach, test evidence, progress and handoff. Product/technical docs,
ADRs and immutable research/evidence describe contracts and facts; they do not
assign or sequence work.

The 82 execution-plan/index/template files, roadmap and first-local-slice sequence
were removed from the active tree. Their full text remains in Git history; remaining
scope was reconciled into GitHub issues. Root AGENTS instructions no longer repeat
a growing session/status journal. No synchronization or replacement tracker exists.

The original hidden planning/seed material stays byte-for-byte in the
[historical archive](archive/README.md), with its original manifest and IDs. The
initial [issue mapping](archive/issue-provenance/README.md) is historical provenance
only. Measured source snapshots/results retain exact hashes and are not rewritten
as new observations. Historical links may describe old trees; agents must not
follow their obsolete instructions as current work.

See [the workflow](workflow.md) and [contribution rules](../CONTRIBUTING.md) for the
issue → test-first behavior change → closing PR → verified merge/closure loop.
No execution plan, root TODO/BACKLOG/STATE/ROADMAP file or agent progress journal
may reintroduce a local work tracker. Private saved playground data remains intact;
this migration changes no running host or BBS runtime contract.
