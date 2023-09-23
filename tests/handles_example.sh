#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

test_container_engine() {
  local -r container_engine="$1"

  cd example

  docker compose down

  "${WHEELSTICKS}" --container-engine "${container_engine}" \
    deploy --wait --wait-timeout 30

  while true; do
    curl --fail --max-time 0.2 --silent http://localhost:8080 || echo "Error $?"
    sleep 0.01s
  done >test.log &

  sleep 2s

  HI_VERSION=B "${WHEELSTICKS}" --container-engine "${container_engine}" \
    deploy --wait --wait-timeout 30

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
