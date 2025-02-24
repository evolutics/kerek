#!/bin/bash

set -o errexit -o nounset -o pipefail

test_container_engine() {
  echo 'Provisioning VM.'
  if [[ -f ssh_config ]]; then
    vagrant snapshot pop
  else
    vagrant ssh-config --host staging >ssh_config
    kerek --container-engine podman \
      provision --force --ssh-config ssh_config staging
    vagrant snapshot push
  fi

  echo 'Building buildable container images.'
  docker compose build
  echo 'Pulling non-buildable container images.'
  docker compose pull --ignore-buildable

  echo 'Deploying via SSH tunnel.'
  (
    kerek --container-engine podman tunnel-ssh --ssh-config ssh_config staging
    trap 'fuser --kill -TERM kerek.sock' EXIT

    docker compose config --images \
      | kerek --host "unix://${PWD}/kerek.sock" transfer-images -

    kerek --host "unix://${PWD}/kerek.sock" \
      deploy --no-build --pull never --remove-orphans --wait
  )

  echo 'Smoke testing.'
  for port in 8080 8181; do
    [[ "$(curl --fail-with-body --max-time 3 --retry 99 --retry-connrefused \
      --retry-max-time 150 "http://192.168.60.159:${port}" \
      | tee /dev/stderr)" == "Hi from ${port}" ]]
  done
}

main() {
  rm --force ssh_config
  vagrant destroy --force
  trap 'vagrant halt' EXIT
  vagrant up

  CONTAINER_ENGINE=docker test_container_engine
  CONTAINER_ENGINE=podman test_container_engine
}

main "$@"
