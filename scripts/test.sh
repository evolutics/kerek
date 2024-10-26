#!/bin/bash

set -o errexit -o nounset -o pipefail

main() {
  local -r script_folder="$(dirname "$(readlink --canonicalize "$0")")"
  cd "$(dirname "${script_folder}")"

  travel-kit

  rustup component add rustfmt
  cargo fmt --all -- --check

  rustup component add clippy
  cargo clippy --all-features --all-targets -- --deny warnings

  cargo check
  cargo test
}

main "$@"
