#!/bin/sh
# Attempts to write outside package directory — sandbox should block this
echo "escape" > /tmp/ripley-sandbox-evil.txt
echo "fs-escape-attempted"
