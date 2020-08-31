#!/usr/bin/env bash
set -euo pipefail
WORK_DIR="$(pwd)"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

$(cd "$WORK_DIR/rust" && cargo build)

source="$WORK_DIR/rust/target/debug/libffi_space.so"

cp -v $source "$WORK_DIR/unity-space/Assets/Plugins"

# copy to library
# cp -v $source "$WORK_DIR/build/01_Data/Plugins/librustlib.so"
