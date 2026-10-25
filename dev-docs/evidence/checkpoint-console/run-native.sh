#!/bin/sh
PATH=/usr/pkg/bin:/usr/bin:/bin:/usr/sbin:/sbin
export PATH
cd /home/factionbuild/faction205 || exit 99
OPENSSL_INCLUDE_DIR=/usr/include
OPENSSL_LIB_DIR=/usr/lib
CARGO_TARGET_DIR=/home/factionbuild/target205
CARGO_BUILD_JOBS=2
export OPENSSL_INCLUDE_DIR OPENSSL_LIB_DIR CARGO_TARGET_DIR CARGO_BUILD_JOBS
run() { name="$1"; shift; "$@" >"$name.log" 2>&1; code=$?; echo "$code" >"$name.exit"; }
run composed cargo test -p faction-client -p faction-omemo --features faction-client/native-mail --all-targets --release --locked --offline -- --test-threads=1
run build cargo build -p faction-client --features native-mail --bin faction-client --example checkpoint_ui_fixture --release --locked --offline
if test "$(cat build.exit)" = 0; then
 mkdir -p bin
 cp /home/factionbuild/target205/release/faction-client bin/faction-client-native
 cp /home/factionbuild/target205/release/examples/checkpoint_ui_fixture bin/checkpoint_ui_fixture
 run ui /usr/pkg/bin/python3.13 tools/probe_checkpoint_ui.py --client /home/factionbuild/faction205/bin/faction-client-native --fixture /home/factionbuild/faction205/bin/checkpoint_ui_fixture --owned-root /home/factionbuild/faction205/private-ui --output /home/factionbuild/faction205/ui-results.json
fi
run disabled cargo test -p faction-omemo --no-default-features --release --locked --offline
run default_build cargo build -p faction-client --release --locked --offline
run vault /usr/pkg/bin/python3.13 tools/probe_native_mail.py --client /home/factionbuild/target205/release/faction-client --output /home/factionbuild/faction205/vault-results.json
run picker /usr/pkg/bin/python3.13 tools/smoke_local_vault_launcher.py /home/factionbuild/target205/release/faction-client --output /home/factionbuild/faction205/picker-results.json
echo complete > complete
