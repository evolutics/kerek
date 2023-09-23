#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

test_container_engine() {
  cd example

  docker compose down

  wheelsticks_deploy

  while true; do
    curl --fail --max-time 0.2 --silent http://localhost:8080 || echo "Error $?"
    sleep 0.01s
  done >test.log &

  sleep 2s

  HI_VERSION=B wheelsticks_deploy

  sleep 2s

  kill %%

  for hi_version in 'A' 'B'; do
    if ! grep --quiet "Hi from ${hi_version}" test.log; then
      echo "No successful ping for hi version: ${hi_version}" >&2
      exit 1
    fi
  done

  local -r ping_errors="$(grep Error test.log)"
  if [[ -n "${ping_errors}" ]]; then
    printf 'Failed pings:\n%s\n' "${ping_errors}" >&2
    exit 1
  fi

  docker compose down
}

wheelsticks_deploy() {
  "${WHEELSTICKS}" --container-engine "${WHEELSTICKS_CONTAINER_ENGINE}" deploy \
    --wait --wait-timeout 30
}

main() {
  (
    WHEELSTICKS_CONTAINER_ENGINE=docker test_container_engine
  )

  (
    export DOCKER_HOST="unix://${PWD}/podman.sock"
    podman system service --time 0 "${DOCKER_HOST}" &
    # shellcheck disable=SC2064
    trap "kill -SIGINT $!" EXIT
    sleep 2s

    WHEELSTICKS_CONTAINER_ENGINE=podman test_container_engine
  )
}

main "$@"
