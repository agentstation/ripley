#!/bin/sh
# Attempts network access — sandbox should block this.
# Exits non-zero if both curl and wget fail (the expected behavior under
# a network-deny sandbox like bwrap --unshare-net or sandbox-exec deny network).
if curl -sS -o /dev/null --max-time 5 http://example.com/test 2>&1; then
  echo "network-allowed-curl"
  exit 0
fi
if wget -q --timeout=5 -O /dev/null http://example.com/test 2>&1; then
  echo "network-allowed-wget"
  exit 0
fi
echo "network-blocked"
exit 1
