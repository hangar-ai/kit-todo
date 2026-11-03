#!/bin/sh
# Builds the native tab (native/, Rust) to WebAssembly and puts the module
# where hangar-kit.json's `entry` says: tabs/todo.wasm. Hangar runs the .wasm
# a kit ships and never builds one, so the module is committed with the source
# that made it: run this in the commit that changes native/.
#
#   scripts/build-native.sh          build the module
#   scripts/build-native.sh --hash   print the hash of the sources, and stop
#
# Needs `rustup target add wasm32-unknown-unknown` once. The hash of the
# sources (native/Cargo.toml, Cargo.lock and src/) goes into the module, and
# scripts/check-native.sh reads it back to tell a module that is stale.
set -eu
cd "$(dirname "$0")/.."

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi
}

source_hash() {
  {
    for file in native/Cargo.toml native/Cargo.lock $(find native/src -type f | LC_ALL=C sort); do
      printf '%s\n' "$file"
      cat "$file"
    done
  } | sha256 | cut -d' ' -f1
}

if [ "${1:-}" = "--hash" ]; then
  source_hash
  exit 0
fi

KIT_TODO_SOURCE="$(source_hash)"
export KIT_TODO_SOURCE
export CARGO_TARGET_DIR="$PWD/native/target"
# The module keeps the paths of the code that made it (in panic messages): the
# machine's own are replaced, so that nothing personal is committed and two
# machines write the same bytes.
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
sysroot="$(rustc --print sysroot)"
export RUSTFLAGS="--remap-path-prefix=$PWD=/kit --remap-path-prefix=$cargo_home=/cargo --remap-path-prefix=$sysroot=/rust ${RUSTFLAGS:-}"

cargo build --manifest-path native/Cargo.toml --release --locked --target wasm32-unknown-unknown
cp native/target/wasm32-unknown-unknown/release/kit_todo_tab.wasm tabs/todo.wasm
echo "tabs/todo.wasm: $(wc -c < tabs/todo.wasm) bytes, sources $KIT_TODO_SOURCE"
