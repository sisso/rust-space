#!/bin/bash
set -euo pipefail
WORK_DIR="$(pwd)"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

${SCRIPT_DIR}/build.sh

LIB_PATH="$WORK_DIR/unity-space/Assets/Plugins/lib"
export LD_LIBRARY_PATH="${LIB_PATH}"
echo "export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}"
ls $LD_LIBRARY_PATH

[[ $(rm -v /tmp/ffi.log) ]] || true
unity-editor -projectPath unity-space $@
