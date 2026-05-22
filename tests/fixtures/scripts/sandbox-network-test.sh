#!/bin/sh
# Attempts network access — sandbox should block this
curl -s http://example.com/test || wget -q http://example.com/test -O /dev/null
echo "network-attempted"
