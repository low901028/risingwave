#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

echo "--- Install required tools"
curl -sSL "https://github.com/bufbuild/buf/releases/download/v1.4.0/buf-$(uname -s)-$(uname -m).tar.gz" | \
tar -xvzf - -C /usr/local --strip-components 1

echo "--- Check protobuf code format && Lint protobuf"
cd proto
buf format -d --exit-code
buf lint

