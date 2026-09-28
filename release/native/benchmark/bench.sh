#!/usr/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RELEASE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$RELEASE_DIR"

echo "[1/3] build rts"
set +e
./build.sh 64 false
set -e
if [[ ! -x "$RELEASE_DIR/rts" ]]; then
  echo "error: release/rts missing after build.sh" >&2
  exit 1
fi

echo "[2/3] build libbench.so"
clang -shared -fPIC -O2 -o "$SCRIPT_DIR/libbench.so" "$SCRIPT_DIR/bench.c"

echo "[3/3] measure"
export RTS="$RELEASE_DIR/rts"
export RUNS="${RUNS:-100}"
echo
echo "Runs: $RUNS"
python3 "$SCRIPT_DIR/bench.py"
