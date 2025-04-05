#!/usr/bin/env bash
set -euo pipefail

build_rust() {
  pushd rust/space-godot
  cargo build -p space-godot || echo "fail to compile, skipping"
  popd
}

while true
do
    build_rust
    godot godot-space/project.godot
done
