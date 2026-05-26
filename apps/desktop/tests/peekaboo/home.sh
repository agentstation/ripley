#!/usr/bin/env bash
# Capture the default React shell (no guard event).
# Requires: peekaboo installed; Ripley running with the window visible
# (toggle with Cmd+Shift+R if it's hidden).

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$DIR/snapshots"
mkdir -p "$OUT"

peekaboo image \
  --app Ripley \
  --window-title Ripley \
  --path "$OUT/home-$(date +%Y%m%d-%H%M%S).png"

echo "wrote $OUT/home-*.png"
