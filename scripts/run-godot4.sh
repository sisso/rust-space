#!/usr/bin/env bash
set -euo pipefail

build_rust() {
  pushd rust/space-godot
  cargo build -p space-godot || echo "fail to compile, skipping"
  popd
}

build_rust
godot4 godot-space/project.godot
