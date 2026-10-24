# Contributing to Faction BBS

All work starts in [GitHub Issues](https://github.com/gonz0w/faction-bbs/issues).
Read [the engineering policy](PROJECT.md), [pinned method](.agent-temper/METHOD.md),
[the issue-only workflow](dev-docs/workflow.md), [product contracts](docs/project.md)
and [privacy/security](docs/security-privacy.md). Research and feature requests are
not promises of support or authorization to expose a board publicly.

1. Find or create a finite issue with scope, non-goals, acceptance, dependencies
   and concrete tests. Split a large epic into executable child issues first.
2. Put the approach/handoff in issue comments, then use an issue-named branch.
   No repository plan, roadmap, TODO list or agent progress journal.
3. For behavior changes, write and run meaningful failing acceptance tests, record
   red, implement to green, then refactor. Documentation/research uses honest named
   validation; never invent TDD. File discovered bugs/scope before unrelated fixes.
4. Run relevant tests and repository checks, update affected docs/README/changelog
   and preserve previously executed source evidence before changing its inputs.
5. Open a PR targeting main with at least one `Closes #NUMBER` line, completed
   issue criteria, acceptance-to-test mapping and actual red/green/refactor evidence.
   Merge after exact-head required checks and resolved review; verify issue closure
   and reconcile parent progress. An abandoned PR leaves unfinished issues open.

Product/member/sysop documentation lives in `docs/`; technical contracts, decisions,
research and validation live in `dev-docs/`. These are documentation, not task state.
Follow the [documentation method](.agent-temper/docs/reference/documentation.md).
Use plain Markdown, relative technical links and dated primary sources. Historical
archives/measurements stay immutable and are not current work instructions.

Rust, primary NetBSD and no JavaScript/TypeScript anywhere—including transitive,
browser, tooling and CI dependencies—remain. Keep approved local membership,
classic/plain terminal access and member-device private-mail keys. Use Python
stdlib/shell for repository tooling. No new cloud agent service is required.

Do not publish private endpoints, credentials, member content or unlicensed game/art
assets. Report sensitive findings via [SECURITY.md](SECURITY.md). Contributions use
[BSD-3-Clause](LICENSE); third-party material retains its own terms.
