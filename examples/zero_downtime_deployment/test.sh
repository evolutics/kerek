#!/bin/bash

set -o errexit -o nounset -o pipefail

test_container_engine() {
  (
    docker compose down
    trap 'docker compose down' EXIT

    GREET_VERSION=A kerek_deploy

    while true; do
      curl --fail --max-time 0.2 --silent localhost:8080 || echo "Error $?"
      sleep 0.01s
    done >test.log &

    sleep 2s

    GREET_VERSION=B kerek_deploy

    sleep 2s

    kill %%

    for greet_version in 'A' 'B'; do
      if ! grep --quiet "^Hi from ${greet_version}$" test.log; then
        echo "No successful ping for greet version: ${greet_version}"
        exit 1
      fi
    done

    local -r ping_errors="$(grep --invert-match '^Hi from ' test.log)"
    if [[ -n "${ping_errors}" ]]; then
      printf 'Failed pings:\n%s\n' "${ping_errors}"
      exit 1
    fi
  )
}

kerek_deploy() {
  kerek deploy --wait --wait-timeout 30
}

main() {
  CONTAINER_ENGINE=docker test_container_engine
  CONTAINER_ENGINE=podman test_container_engine
}

main "$@"
