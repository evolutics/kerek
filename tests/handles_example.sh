#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

test_container_engine() {
  local -r container_engine="$1"

  cd example

  docker compose down

  "${WHEELSTICKS}" --container-engine "${container_engine}" deploy --wait \
    --wait-timeout 60

  curl --fail --max-time 3 --retry 99 --retry-connrefused --retry-max-time 15 \
    --show-error http://localhost:8080

  docker compose down
}

main() {
  (
    test_container_engine docker
  )

  (
    export DOCKER_HOST="unix://${PWD}/podman.sock"
    podman system service --time 0 "${DOCKER_HOST}" &
    # shellcheck disable=SC2064
    trap "kill -SIGINT $!" EXIT
    sleep 2s

    test_container_engine podman
  )
}

main "$@"
