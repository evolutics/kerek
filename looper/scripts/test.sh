#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

check_general_cleanliness() {
  docker run --entrypoint sh --rm --volume "${PWD}:/workdir" \
    evolutics/travel-kit:0.8.0 -c \
    'git ls-files -z | xargs -0 travel-kit check --'
}

main() {
  local -r script_folder="$(dirname "$(readlink --canonicalize "$0")")"
  cd "$(dirname "${script_folder}")"

  check_general_cleanliness
}

main "$@"
