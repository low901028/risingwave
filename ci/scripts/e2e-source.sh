#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

echo "--- Adjust permission"
chmod +x ./target/debug/risingwave
chmod +x ./target/debug/risedev-playground
chmod +x ./target/debug/risingwave_regress_test

echo "--- Generate RiseDev CI config"
cp risedev-components.ci.env risedev-components.user.env

echo "--- Prepare RiseDev playground"
~/cargo-make/makers pre-start-playground
~/cargo-make/makers link-all-in-one-binaries

echo "--- e2e test w/ Rust frontend - source with kafka"
~/cargo-make/makers clean-data
~/cargo-make/makers ci-start ci-kafka
./scripts/source/prepare_ci_kafka.sh
sqllogictest -p 4566 './e2e_test/source/**/*.slt'