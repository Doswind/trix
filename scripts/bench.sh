#!/usr/bin/env bash
# Offscreen render benchmark for every effect.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release
SIZE="${SIZE:-160x48}"
FRAMES="${FRAMES:-300}"
EFFECTS=(
    rain fire dragon map matrix blackhole beams fireworks solar galaxy buddha
    tomjerry mario ecg dandelion fish landscape shanghai tiananmen traffic
    tornado storm monalisa ballet demoneye life
)
for effect in "${EFFECTS[@]}"; do
    echo "================ $effect ================"
    ./target/release/trix "$effect" --bench --size "$SIZE" --frames "$FRAMES" --fps 30
done
