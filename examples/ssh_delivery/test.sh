#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

test_container_engine() {
  if [[ -f ssh_config ]]; then
    vagrant snapshot pop
  else
    vagrant ssh-config --host ssh-host >ssh_config
    kerek --container-engine podman \
      provision --force --ssh-config ssh_config ssh-host
    vagrant snapshot push
  fi

  docker compose pull --ignore-buildable

  (
    kerek --container-engine podman tunnel-ssh --ssh-config ssh_config ssh-host
    trap 'kill "$(lsof -t "${PWD}/kerek.sock")"' EXIT

    docker compose config --images \
      | kerek --host "unix://${PWD}/kerek.sock" transfer-images -

    kerek --host "unix://${PWD}/kerek.sock" \
      deploy --no-build --pull never --remove-orphans --wait
  )

  [[ "$(curl --fail-with-body --max-time 3 --retry 99 --retry-connrefused \
    --retry-max-time 150 http://192.168.60.159 \
    | tee /dev/stderr)" == 'hello-world' ]]
}

main() {
  vagrant destroy --force
  trap 'vagrant halt' EXIT
  vagrant up

  (
    trap 'rm --force ssh_config' EXIT

    CONTAINER_ENGINE=docker test_container_engine

    (
      export DOCKER_HOST="unix://${PWD}/podman.sock"
      podman system service --time 0 "${DOCKER_HOST}" &
      trap 'kill "$(lsof -t "${PWD}/podman.sock")"' EXIT

      CONTAINER_ENGINE=podman test_container_engine
    )
  )
}

main "$@"
