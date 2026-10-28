# Reproduce the native attachment observation

Use the reviewed source revision on NetBSD 11 ARM64 with native Rust/Cargo 1.96.0,
base OpenSSL 3, Python 3.13 and the already preflighted locked dependency cache.
These commands produce **a new observation** with its own binary/result identities.
Keep the historical records unchanged. No installed provider, host listener or
saved playground instance is needed; both synthetic homes stay on guest loopback.

Choose a fresh owned private root in persistent guest storage. Keep the source
checkout, dependency cache, target and proof root across the runtime restart;
[NetBSD 11 defaults](https://raw.githubusercontent.com/NetBSD/src/netbsd-11/etc/defaults/rc.conf)
clear `/tmp` at boot. Do not change service settings to retain it.
Record these absolute source/root/target values and reuse them after the restart. Source inputs must be regular single-link
files without group/other writes; extract Git archives with a private umask or
narrow permissions on your owned staged files before collection.

```sh
set -eu
umask 077
FACTION_QUAL_SOURCE="$(pwd -P)"
FACTION_QUAL_ROOT="$(mktemp -d "$HOME/faction-native-attachments.XXXXXX")"
FACTION_QUAL_ROOT="$(cd "$FACTION_QUAL_ROOT" && pwd -P)"
export FACTION_QUAL_SOURCE FACTION_QUAL_ROOT
printf 'Owned native NetBSD attachment qualification\n' > "$FACTION_QUAL_ROOT/OWNED"
export CARGO_TARGET_DIR="$FACTION_QUAL_ROOT/target"
export CARGO_BUILD_JOBS=2
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_LIB_DIR=/usr/lib
export OPENSSL_NO_VENDOR=1
```

Build with two CPUs/2 GiB and at most two compiler jobs. The existing FFI build
rejects cross compilation. Keep the normal Cargo cache or explicitly set
`CARGO_HOME` to the preflighted cache; do not silently fetch missing dependencies.

```sh
cargo test -p faction-client -p faction-omemo -p faction-files \
  --features faction-client/network-mail --all-targets --release --locked --offline --no-run
cargo build -p faction-client --features network-mail --bin faction-client \
  --example mail_blob_fixture --example mail_reply_fixture --release --locked --offline
mkdir -m 700 "$FACTION_QUAL_ROOT/bin"
cp "$CARGO_TARGET_DIR/release/faction-client" "$FACTION_QUAL_ROOT/bin/faction-client"
cp "$CARGO_TARGET_DIR/release/examples/mail_blob_fixture" "$FACTION_QUAL_ROOT/bin/mail_blob_fixture"
cp "$CARGO_TARGET_DIR/release/examples/mail_reply_fixture" "$FACTION_QUAL_ROOT/bin/mail_reply_fixture"
chmod 700 "$FACTION_QUAL_ROOT/bin/"*
```

Shut down the disposable build guest and restart it with **two CPUs/1 GiB**.
Restore the recorded path/environment values above without creating a different
root, and return to the same source directory. Verify `sysctl -n hw.ncpu
hw.physmem64 hw.machine_arch` before running:

```sh
cargo test -p faction-client -p faction-omemo -p faction-files \
  --features faction-client/network-mail --all-targets --release --locked --offline -- --test-threads=1
PYTHONDONTWRITEBYTECODE=1 python3.13 -m unittest discover -s tools -p test_collect_attachment_netbsd.py
mkdir -m 700 "$FACTION_QUAL_ROOT/results" "$FACTION_QUAL_ROOT/tmp" "$FACTION_QUAL_ROOT/console"
printf 'Owned native attachment console proof\n' > "$FACTION_QUAL_ROOT/console/OWNED"
export TMPDIR="$FACTION_QUAL_ROOT/tmp"
export PYTHONDONTWRITEBYTECODE=1
python3.13 tools/probe_attachment_transfer.py \
  --binary "$FACTION_QUAL_ROOT/bin/mail_blob_fixture" --openssl /usr/bin/openssl \
  --output "$FACTION_QUAL_ROOT/results/transfer.json"
python3.13 tools/probe_mail_attachments.py \
  --client "$FACTION_QUAL_ROOT/bin/faction-client" \
  --fixture "$FACTION_QUAL_ROOT/bin/mail_blob_fixture" \
  --reply-fixture "$FACTION_QUAL_ROOT/bin/mail_reply_fixture" \
  --owned-root "$FACTION_QUAL_ROOT/console" --openssl /usr/bin/openssl \
  --output "$FACTION_QUAL_ROOT/results/console.json"
```

Construct the collector's exact schema-1 input manifest from the selected source
set, materialized executables and freshly produced result bytes. The manifest
binds observations; it does not independently prove they executed.

```sh
PYTHONPATH=tools python3.13 - <<'PY'
import hashlib, json, os
from pathlib import Path
from collect_attachment_netbsd import SOURCE_FILES
source = Path(os.environ['FACTION_QUAL_SOURCE'])
root = Path(os.environ['FACTION_QUAL_ROOT'])
roles = ('faction-client', 'mail_blob_fixture', 'mail_reply_fixture')
def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()
manifest = {
    'schema_version': 1,
    'files': {name: digest(source/name) for name in SOURCE_FILES},
    'binaries': {name: digest(root/'bin'/name) for name in roles},
    'results': {name: digest(root/'results'/(name+'.json')) for name in ('transfer', 'console')},
}
path = root/'results/inputs.json'
path.write_text(json.dumps(manifest, indent=2)+'\n')
path.chmod(0o600)
PY
python3.13 tools/collect_attachment_netbsd.py \
  --source-root "$FACTION_QUAL_SOURCE" \
  --inputs-manifest "$FACTION_QUAL_ROOT/results/inputs.json" \
  --transfer-result "$FACTION_QUAL_ROOT/results/transfer.json" \
  --console-result "$FACTION_QUAL_ROOT/results/console.json" \
  --client "$FACTION_QUAL_ROOT/bin/faction-client" \
  --blob-fixture "$FACTION_QUAL_ROOT/bin/mail_blob_fixture" \
  --reply-fixture "$FACTION_QUAL_ROOT/bin/mail_reply_fixture" \
  --output "$FACTION_QUAL_ROOT/results/native-envelope.json"
```

Collection refuses non-NetBSD, wrong resource/architecture, unsafe modes/paths,
source/binary/result mismatches and existing output. Do not edit the old probe's
false qualification fields. Retain new private native command/test/console
captures and separately record compiler, source and library identities. Archive
private evidence deliberately, stop/reap only your disposable processes, verify
forward closure, and remove only your marked test assets. The saved owner
playground and earlier evidence remain intact.
