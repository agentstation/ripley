#!/bin/sh
# Benign operations within the package directory — sandbox should allow this
mkdir -p ./sandbox-test-output
echo "hello" > ./sandbox-test-output/test.txt
echo "benign-complete"
