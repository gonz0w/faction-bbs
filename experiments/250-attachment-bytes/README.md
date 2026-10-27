# Attachment byte experiment

This laboratory compares bounded historical AES-256-GCM bytes with an unchanged
Gajim reference. It is not product code or an attachment service. See the
[research report](../../dev-docs/research/0059-attachment-byte-proof.md) for the
observed results, test history, dependency identities and limitations.

The stdlib unit contracts can run without the external crypto inputs:

```sh
python3 -m unittest discover -s experiments/250-attachment-bytes -p test_proof.py
```

A fresh native/reference run requires the exact qualified native library and
binary, unchanged external Gajim module and isolated Python environment described
in the report and recorded qualification. Do not copy the GPL reference, wheels or
binaries into Git. Stop and record missing inputs; do not substitute a different
version and claim reproduction. Build only inside an owned disposable fixture and
use explicit paths. The recorded native command linked `adapter.c` against the
qualified static `libcrypto.a` with `clang -std=c11 -O2 -Wall -Wextra -Werror`.

With those inputs prepared, select a new result path and a spool path that does
not exist. For example, with `FACTION250_ROOT` set to the owned fixture root:

```sh
"$FACTION250_ROOT/venv/bin/python" -I -B experiments/250-attachment-bytes/proof.py \
  --native-binary "$FACTION250_ROOT/phase2/adapter" \
  --gajim-module "$FACTION250_ROOT/downloads/gajim-aes.py" \
  --spool "$FACTION250_ROOT/new-spool" --result "$FACTION250_ROOT/new-result.json"
```

The recorded integration and native test runs used these qualified environment
paths, with `FACTION250_ROOT` identifying the owned root:

```sh
export TMPDIR="$FACTION250_ROOT/phase2/tmp"
export OPENSSL_CONF="$FACTION250_ROOT/openssl-empty.cnf"
export OPENSSL_MODULES="$FACTION250_ROOT/modules-empty"
export OPENSSL_ENGINES="$FACTION250_ROOT/engines-empty"
python3 -s -B experiments/250-attachment-bytes/test_proof.py \
  --native-binary "$FACTION250_ROOT/phase2/adapter"
```

Set the same environment before running the integration example above. The native
test command uses `-s -B` so the sibling `proof.py` remains importable. Inspect
and remove only the owned fixture after review, recording actual cleanup. Inputs are fixed public lab
values. No production keys, member content or credentials belong in this experiment.
The saved-evidence check and stdlib tests do not rerun the real crypto comparison.
