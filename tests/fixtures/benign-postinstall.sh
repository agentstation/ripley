#!/bin/sh
# Typical benign postinstall: compile native addon.
# Used as a test fixture for the static analyzer — should score low risk.

node-gyp rebuild
echo "Native module compiled successfully."
