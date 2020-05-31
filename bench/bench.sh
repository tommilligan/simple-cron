#!/bin/bash
# This script requires:
# - a release build of the binary to be available
# - hyperfine to be installed (cargo install hyperfine)

set -e

echo >&2 "Generating benchmark input"
rm -rf bench/bench-input-large.lines
for i in {1..4000}; do cat bench/bench-input.lines; done > bench/bench-input-large.lines

echo >&2 "Running benchmark"
hyperfine 'cat bench/bench-input-large.lines | ./target/release/scron 09:00'
