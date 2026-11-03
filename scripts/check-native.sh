#!/bin/sh
# Fails when the committed module, tabs/todo.wasm, was not built from the
# sources that are committed beside it: the hash scripts/build-native.sh puts
# in the module must be the hash of native/Cargo.toml, Cargo.lock and src/ now.
# Needs no Rust. Fix: scripts/build-native.sh, and commit the module.
set -eu
cd "$(dirname "$0")/.."

module=tabs/todo.wasm
[ -f "$module" ] || { echo "$module is missing: run scripts/build-native.sh" >&2; exit 1; }
hash="$(scripts/build-native.sh --hash)"
if grep -aq "$hash" "$module"; then
  echo "$module is built from the sources ($hash)"
else
  echo "$module is stale: it was not built from the sources in native/ ($hash)." >&2
  echo "Run scripts/build-native.sh and commit tabs/todo.wasm with the source." >&2
  exit 1
fi
