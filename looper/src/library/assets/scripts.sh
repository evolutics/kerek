#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

provision() {
  echo 'Provisioning remote for Podman with Docker connections.' >&2
  wheelsticks provision --force --ssh-config "${KEREK_SSH_CONFIG}" -- \
    "${KEREK_ENVIRONMENT_ID}"
}

build() {
  echo 'Building with Docker Compose.' >&2
  docker compose build

  echo 'Pulling remaining images.' >&2
  docker compose pull --ignore-buildable
}

deploy() {
  echo 'Transferring images.' >&2
  docker compose config --images | wheelsticks run-with-ssh-config -- \
    "${KEREK_SSH_CONFIG}" wheelsticks --host "ssh://${KEREK_ENVIRONMENT_ID}" \
    transfer-images -

  echo 'Deploying containers on remote.' >&2
  wheelsticks run-with-ssh-config -- "${KEREK_SSH_CONFIG}" docker \
    --host "ssh://${KEREK_ENVIRONMENT_ID}" compose up --detach --no-build \
    --pull never --remove-orphans --wait
}

env_tests() {
  echo 'No env tests. Continuing.' >&2
}

move_to_next_version() {
  while true; do
    git fetch --prune

    child_commit="$(git rev-list --ancestry-path --first-parent \
      HEAD.."${KEREK_GIT_BRANCH}" | tail -1)"

    if [[ -n "${child_commit}" ]]; then
      echo "Checking out Git commit ${child_commit}." >&2
      git checkout "${child_commit}"
      break
    fi

    sleep "$(("${RANDOM}" % 20))s"
  done
}

main() {
  "$1"
}

main "$@"
