#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

check_general_cleanliness() {
  travel-kit
}

test_rust() {
  rustup component add rustfmt
  cargo fmt --all -- --check

  rustup component add clippy
  cargo clippy --all-features --all-targets -- --deny warnings

  cargo check
  cargo test
}

main() {
  local -r script_folder="$(dirname "$(readlink --canonicalize "$0")")"
  cd "$(dirname "${script_folder}")"

  check_general_cleanliness
  test_rust
}

main "$@"
