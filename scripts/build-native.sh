#!/bin/sh
# Builds the native tab (native/) to WebAssembly and puts it where the
# manifest's `entry` names it: tabs/annotate.wasm, committed with the source
# that made it. Hangar runs the module a kit ships and never builds one.
#
#   scripts/build-native.sh          build it
#   scripts/build-native.sh --check  build it, and fail if the committed one is not that
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
built=native/target/wasm32-unknown-unknown/release/kit_annotate_tab.wasm
if [ "${1:-}" = "--check" ]; then
  if cmp -s "$built" tabs/annotate.wasm; then
    echo "tabs/annotate.wasm is what native/ builds"
    exit 0
  fi
  echo "tabs/annotate.wasm is stale: run scripts/build-native.sh and commit the result" >&2
  exit 1
fi
cp "$built" tabs/annotate.wasm
echo "tabs/annotate.wasm: $(wc -c < tabs/annotate.wasm) bytes"
