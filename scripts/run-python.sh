#!/usr/bin/env bash
set -euo pipefail
WORK_DIR="$(pwd)"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

$(cd rust && cargo build)
python python/test1.py
