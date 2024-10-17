#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
WORKING_DIR="$(pwd)"

build_rust() {
  pushd "$SCRIPT_DIR/../rust/space-godot"
  cargo build -p space-godot || echo "fail to compile, skipping"
  popd
}

build_rust

pushd "$SCRIPT_DIR/.."
godot4 godot-space/project.godot
