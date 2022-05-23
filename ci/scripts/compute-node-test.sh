#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

echo "--- Install llvm-tools-preview, clippyllvm-cov, nextest"
rustup component add llvm-tools-preview clippy

cargo install cargo-llvm-cov
curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin

echo "--- Run rust clippy check"
cargo clippy --all-targets --all-features --locked -- -D warnings

echo "--- Build documentation"
cargo doc --document-private-items --no-deps

echo "--- Run rust failpoints test"
cargo doc --document-private-items --no-deps

echo "--- Run rust doc check"
cargo test --doc

echo "--- Run rust test with coverage"
cargo llvm-cov nextest --lcov --output-path lcov.info -- --no-fail-fast


