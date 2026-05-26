#!/usr/bin/env bash
# Fire a synthetic guard prompt and capture the rendered dialog.
# Requires: peekaboo installed; Ripley running; guard-bench built.

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$DIR/snapshots"
mkdir -p "$OUT"

BENCH_BIN="$(git rev-parse --show-toplevel)/target/release/guard-bench"
if [ ! -x "$BENCH_BIN" ]; then
  echo "guard-bench not built; run: cargo build -p ripley-desktop --release"
  exit 2
fi

# Fire 1 event in background; capture window once the dialog appears.
"$BENCH_BIN" 1 &
BENCH_PID=$!

sleep 0.5

peekaboo image \
  --app Ripley \
  --window-title Ripley \
  --path "$OUT/dialog-$(date +%Y%m%d-%H%M%S).png"

wait "$BENCH_PID" 2>/dev/null || true
echo "wrote $OUT/dialog-*.png"
