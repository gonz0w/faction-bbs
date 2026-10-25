#!/bin/sh
PATH=/usr/pkg/bin:/usr/bin:/bin:/usr/sbin:/sbin
export PATH
cd /home/factionbuild/faction208/probe || exit99
OPENSSL_INCLUDE_DIR=/usr/include
OPENSSL_LIB_DIR=/usr/lib
CARGO_TARGET_DIR=/home/factionbuild/target208
CARGO_BUILD_JOBS=2
CARGO_HOME=/home/factionbuild/faction208/cargo-home
export OPENSSL_INCLUDE_DIR OPENSSL_LIB_DIR CARGO_TARGET_DIR CARGO_BUILD_JOBS CARGO_HOME
cargo test --release --locked --offline >../tests.log 2>&1
echo $? >../tests.exit
cargo build --release --locked --offline >../build.log 2>&1
echo $? >../build.exit
echo complete >../complete
