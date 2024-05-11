#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

provision() {
  ssh -F "${KEREK_SSH_CONFIGURATION}" "${KEREK_SSH_HOST}" \
    <"${KEREK_CACHE_FOLDER}/provision_on_remote.sh"
}

build() {
  docker compose build
}

deploy() {
  # TODO: Transfer built images via SSH as in
  # `docker save -- … | docker --host "ssh://${KEREK_SSH_HOST}" load`.

  local -r custom_ssh_folder="${PWD}/${KEREK_CACHE_FOLDER}/custom_ssh"
  chmod +x -- "${custom_ssh_folder}/ssh"
  local -r real_ssh="$(which ssh)"

  PATH="${custom_ssh_folder}:${PATH}" REAL_SSH="${real_ssh}" docker --host \
    "ssh://${KEREK_SSH_HOST}" compose up --detach --remove-orphans --wait
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
