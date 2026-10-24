## Change and reason

Describe the issue's concrete problem and resulting behavior.

## Closing issues

Closes #NUMBER

Use one real same-repository open executable issue per line. Finish its acceptance
criteria before merge. An epic/reference alone does not qualify; create a bounded
child before implementing partial scope. Do not close issues for unmerged work.

## Acceptance and tests

Map every closing issue's criteria to named tests, commands, actual results and
relevant evidence. Include negative/failure/classic fallback checks when applicable.
State unavailable checks and support limits honestly. Check the issue criteria only
after evidence supports them; leave its GitHub state open until merge.

## TDD evidence

Mode: behavior

Red: Replace with the actual command and observed failure before implementation.
Green: Replace with the passing command and outcome after the implementation.
Refactor: Replace with the cleanup performed and the tests that remained green.

For prose-only changes use Mode: docs or Mode: research plus Reason: explaining
actual validation. Executable/configuration changes require behavior tests. Never
invent red-first history; the fixed pre-#181 exception is documented in the workflow.

## Review

- [ ] Finished the linked issues' agreed scope; captured new bugs/follow-ups in GitHub.
- [ ] Reviewed README and relevant product/technical docs/changelog for actual behavior.
- [ ] Ran required repository checks and relevant tests; preserved historical evidence.
- [ ] Preserved local approval, device keys, classic fallback and resource boundaries.
- [ ] Added no JS/TS, secret data, real private endpoints or unlicensed assets.
- [ ] Added no local task tracker, execution plan or agent progress journal.
