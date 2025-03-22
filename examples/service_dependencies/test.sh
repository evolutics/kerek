#!/bin/bash

set -o errexit -o nounset -o pipefail

test_container_engine() {
  (
    trap 'docker compose down' EXIT

    kerek deploy my-service-a
    [[ "$(docker compose ps --services | tee /dev/stderr)" == 'my-service-a
my-service-b' ]]

    docker compose down

    kerek deploy --no-deps my-service-a
    [[ "$(docker compose ps --services | tee /dev/stderr)" == 'my-service-a' ]]

    docker compose down

    kerek deploy my-service-b
    [[ "$(docker compose ps --services | tee /dev/stderr)" == 'my-service-b' ]]
  )
}

main() {
  CONTAINER_ENGINE=docker test_container_engine
  CONTAINER_ENGINE=podman test_container_engine
}

main "$@"
