#!/bin/bash

set -o errexit -o nounset -o pipefail

cd -- "$(dirname -- "$0")/.."

travel-kit

rustup component add rustfmt
cargo fmt --all -- --check

rustup component add clippy
cargo clippy --all-features --all-targets -- --deny warnings

cargo check
cargo test
