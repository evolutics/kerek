#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

provision() {
  >&2 echo 'Provisioning remote for Podman with Docker connections.'
  ssh -F "${KEREK_SSH_CONFIGURATION}" "${KEREK_SSH_HOST}" \
    <"${KEREK_CACHE_FOLDER}/provision_on_remote.sh"
}

build() {
  >&2 echo 'Building with Docker Compose.'
  docker compose build
}

deploy() {
  >&2 echo 'Getting image names from Compose configuration.'
  local images
  mapfile -t images < <(docker compose config --images)
  readonly images

  >&2 echo "Transferring ${#images[@]} images: ${images[*]}"
  docker save -- "${images[@]}" | run_with_ssh_docker_host docker load

  >&2 echo 'Deploying containers on remote.'
  run_with_ssh_docker_host docker compose up --detach --no-build --pull never \
    --remove-orphans --wait
}

run_with_ssh_docker_host() {
  local -r configured_ssh_folder="${PWD}/${KEREK_CACHE_FOLDER}/configured_ssh"
  chmod +x -- "${configured_ssh_folder}/ssh"
  local -r real_ssh="$(which ssh)"

  DOCKER_HOST="ssh://${KEREK_SSH_HOST}" \
    PATH="${configured_ssh_folder}:${PATH}" REAL_SSH="${real_ssh}" "$@"
}

move_to_next_version() {
  while true; do
    git fetch --prune

    child_commit="$(git rev-list --ancestry-path --first-parent \
      HEAD.."${KEREK_GIT_BRANCH}" | tail -1)"

    if [[ -n "${child_commit}" ]]; then
      >&2 echo "Checking out Git commit ${child_commit}."
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
