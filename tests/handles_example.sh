#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

main() {
  export DOCKER_HOST="unix://${PWD}/podman.sock"

  cd example

  podman system service --time 0 "${DOCKER_HOST}" &
  # shellcheck disable=SC2064
  trap "kill -SIGINT $!" EXIT
  sleep 2s

  docker compose down
  docker compose build

  "${WHEELSTICKS}" --container-engine podman deploy

  curl --fail --max-time 3 --retry 99 --retry-connrefused --retry-max-time 15 \
    --show-error http://localhost:8080
}

main "$@"
