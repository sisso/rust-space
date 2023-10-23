#!/usr/bin/env bash
set -euo pipefail

build_rust() {
  pushd rust/space-godot
  cargo build -p space-godot
  popd
}

build_rust
godot4 godot-space/project.godot &
# do not work, at least looks like not
# pushd rust/space-godot
# cargo watch -x 'build -p space-godot'
# popd

while true
do
  read -p "press enter to enter to compile"
  build_rust
done
