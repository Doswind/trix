#!/usr/bin/env bash
# Offscreen render benchmark for every effect.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release
SIZE="${SIZE:-160x48}"
FRAMES="${FRAMES:-300}"
for effect in rain fire dragon map matrix; do
    echo "================ $effect ================"
    ./target/release/texiao "$effect" --bench --size "$SIZE" --frames "$FRAMES" --fps 30
done
