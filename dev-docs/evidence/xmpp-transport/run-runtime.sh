#!/bin/sh
PATH=/usr/pkg/bin:/usr/bin:/bin:/usr/sbin:/sbin
export PATH
cd /home/factionbuild/faction208/probe || exit 99
OPENSSL_INCLUDE_DIR=/usr/include
OPENSSL_LIB_DIR=/usr/lib
CARGO_TARGET_DIR=/home/factionbuild/target208
CARGO_BUILD_JOBS=1
CARGO_HOME=/home/factionbuild/faction208/cargo-home
export OPENSSL_INCLUDE_DIR OPENSSL_LIB_DIR CARGO_TARGET_DIR CARGO_BUILD_JOBS CARGO_HOME
cargo test --release --locked --offline >../runtime-tests.log 2>&1
echo $? >../runtime-tests.exit
cargo build --release --locked --offline >../runtime-build.log 2>&1
echo $? >../runtime-build.exit
cd /home/factionbuild/faction208 || exit 99
/usr/pkg/bin/python3.13 probe_xmpp_transport.py --binary /home/factionbuild/target208/release/faction-xmpp-probe --openssl /usr/bin/openssl --output native-matrix.json >matrix.log 2>&1
echo $? >matrix.exit
/usr/pkg/bin/python3.13 measure_xmpp_transport.py --binary /home/factionbuild/target208/release/faction-xmpp-probe --openssl /usr/bin/openssl --output native-cost.json >cost.log 2>&1
echo $? >cost.exit
echo complete >runtime-complete
