#!/bin/bash

# Exits as soon as any line fails.
set -euo pipefail

while getopts 't:p:' opt; do
    case ${opt} in
        t )
            target=$OPTARG
            ;;
        p )
            profile=$OPTARG
            ;;
        \? )
            echo "Invalid Option: -$OPTARG" 1>&2
            exit 1
            ;;
        : )
            echo "Invalid option: $OPTARG requires an arguemnt" 1>&2
            ;;
    esac
done
shift $((OPTIND -1))

echo "--- Rust cargo-hakari check"
cargo hakari verify

echo "--- Rust format check"
cargo fmt --all -- --check

echo "--- Build Rust components"
cargo build -p risingwave_cmd_all -p risedev -p risingwave_regress_test --profile $profile

echo "--- Compress RisingWave debug info"
objcopy --compress-debug-sections=zlib-gnu target/$target/risingwave

