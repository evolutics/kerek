#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

provision() {
  wheelsticks provision --ssh-configuration "${KEREK_SSH_CONFIGURATION}" -- \
    "${KEREK_SSH_HOST}"
}

build() {
  wheelsticks build
}

deploy() {
  wheelsticks deploy --ssh-configuration "${KEREK_SSH_CONFIGURATION}" -- \
    "${KEREK_SSH_HOST}"
}

move_to_next_version() {
  while true; do
    git fetch --prune

    child_commit="$(git rev-list --ancestry-path --first-parent \
      HEAD.."${KEREK_GIT_BRANCH}" | tail -1)"

    if [[ -n "${child_commit}" ]]; then
      echo "Checking out Git commit ${child_commit}."
      git checkout "${child_commit}"
      break
    fi

    sleep "$(("${RANDOM}" % 20))s"
  done
}

base_test() {
  scripts/base_test.sh
}

smoke_test() {
  scripts/smoke_test.sh
}

acceptance_test() {
  scripts/acceptance_test.sh
}

main() {
  "$1"
}

main "$@"
