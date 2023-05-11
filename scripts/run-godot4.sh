#!/usr/bin/env bash
set -euo pipefail
pushd rust/space-godot
cargo build -p space-godot
popd
godot4 godot-space/project.godot
