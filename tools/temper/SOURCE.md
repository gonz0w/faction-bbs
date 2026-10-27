# Retained Temper selection checker

Source: [gonz0w/agent-temper tools/check_policy.py](https://github.com/gonz0w/agent-temper/blob/781a4b9f71d002b93bc4ad26669d01a3b37c482e/tools/check_policy.py)

Revision: `781a4b9f71d002b93bc4ad26669d01a3b37c482e`. The helper is copied unchanged outside
`.agent-temper`; it is not part of that bundle inventory.

- Helper SHA-256: `e61f9bac92fc5296c3f12182f6490ed957006da36cba9f2900213af864cd2d9c`
- [Complete BSD-3-Clause license](LICENSE), copied from the same revision.
- License SHA-256: `1bc9ce76b659a58c7b8edb88304880f36fd2a005f7b7f3349d1efc860d8deb87`

Run only on an existing authorized source-check executor under local policy.
This command starts no product, native fixture, VM or external service.

From the repository root, with Python 3.11+ standard library and the actual
assessment date (the date below is a synthetic command example):

```sh
python3 tools/temper/check_policy.py --root . --manifest policy.toml --as-of 2026-10-27
```

The JSON report keeps policy conformance, selection approval and execution authority
`unverified`, even on exit 0. It checks declared references, dependencies, conflicts,
cycles and date expiry; it does not interpret prose or event expiry. No CI, hooks,
validators or check-selection behavior is changed. A failure requires inspection
under the existing policy process; it does not permit a bypass or automatic repair.
