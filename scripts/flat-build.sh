#!/usr/bin/env bash

set -euo pipefail

WORK_DIR="$(pwd)"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

flatc=/home/sisso/workspace/others/flatbuffers/flatc

for file in $WORK_DIR/data/*; do
  echo "processing ${file}..."
  $flatc --rust -o "$WORK_DIR/rust/space-domain/src" "${file}"
  # $flatc --csharp -o "$WORK_DIR/mono/test-csharp-rust/" "${file}"
  # $flatc --csharp -o "$WORK_DIR/unity-space/Assets/src" "${file}"
done

echo "complete"
