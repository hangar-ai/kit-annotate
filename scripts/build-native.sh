#!/bin/sh
# Builds the native tab (native/) to WebAssembly and puts it where the
# manifest's `native` names it: tabs/annotate.wasm, committed with the source
# that made it. Hangar runs the module a kit ships and never builds one.
#
# Needs the toolchain rust-toolchain.toml names, with its wasm32 target
# (rustup installs both on the first run). The paths of Cargo's home and of
# this folder are written out of the module, so that a build here and one in
# CI give the same bytes: CI fails when the committed module is stale.
set -eu
cd "$(dirname "$0")/.."
kit="$(pwd)"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="--remap-path-prefix=$cargo_home=/cargo --remap-path-prefix=$kit=/kit" \
  cargo build --manifest-path native/Cargo.toml --release --locked --target wasm32-unknown-unknown
cp native/target/wasm32-unknown-unknown/release/kit_annotate_tab.wasm tabs/annotate.wasm
echo "tabs/annotate.wasm: $(wc -c < tabs/annotate.wasm) bytes"
