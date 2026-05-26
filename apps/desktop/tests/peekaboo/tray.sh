#!/usr/bin/env bash
# Capture the Ripley tray icon in the menu bar.
# Requires: peekaboo installed; Ripley running.

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="$DIR/snapshots"
mkdir -p "$OUT"

peekaboo image \
  --mode screen \
  --path "$OUT/tray-$(date +%Y%m%d-%H%M%S).png" \
  --capture-focus auto

echo "wrote $OUT/tray-*.png"
