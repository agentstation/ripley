#!/bin/sh
# Typical benign postinstall: compile native addon and copy files.
# Used as a test fixture for the static analyzer — should score low risk.

node-gyp rebuild
mkdir -p dist
cp -r src/* dist/
echo "Build complete."
