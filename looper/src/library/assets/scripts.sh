#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

provision() {
  echo 'Provisioning remote for Podman with Docker connections.' >&2
  ssh -F "${KEREK_SSH_CONFIGURATION}" "${KEREK_ENVIRONMENT_ID}" \
    <"${KEREK_CACHE_FOLDER}/provision_on_remote.sh"
}

build() {
  echo 'Building with Docker Compose.' >&2
  docker compose build
}

deploy() {
  echo 'Getting image names from Compose configuration.' >&2
  local images
  mapfile -t images < <(docker compose config --images)
  readonly images

  echo "Transferring ${#images[@]} images: ${images[*]}" >&2
  docker save -- "${images[@]}" | run_with_ssh_docker_host docker load

  echo 'Deploying containers on remote.' >&2
  run_with_ssh_docker_host docker compose up --detach --no-build --pull never \
    --remove-orphans --wait
}

run_with_ssh_docker_host() {
  local -r configured_ssh_folder="${PWD}/${KEREK_CACHE_FOLDER}/configured_ssh"
  chmod +x -- "${configured_ssh_folder}/ssh"
  local -r real_ssh="$(which ssh)"

  DOCKER_HOST="ssh://${KEREK_ENVIRONMENT_ID}" KEREK_REAL_SSH="${real_ssh}" \
    PATH="${configured_ssh_folder}:${PATH}" "$@"
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
