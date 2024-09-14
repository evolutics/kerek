#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

# TODO: Test Podman CLI, too.
main() {
  vagrant destroy --force
  trap 'vagrant destroy --force' EXIT
  vagrant up

  (
    trap 'rm --force ssh_config' EXIT
    vagrant ssh-config --host ssh-host >ssh_config

    kerek provision --force --ssh-config ssh_config ssh-host

    docker compose pull --ignore-buildable

    (
      kerek tunnel-ssh --local-socket temp.sock --ssh-config ssh_config ssh-host
      trap 'kill "$(lsof -t "${PWD}/temp.sock")"' EXIT

      docker compose config --images \
        | kerek --host "unix://${PWD}/temp.sock" transfer-images -

      kerek --host "unix://${PWD}/temp.sock" \
        deploy --no-build --pull never --remove-orphans --wait
    )

    local -r result="$(curl --fail-with-body --max-time 3 --retry 99 \
      --retry-connrefused --retry-max-time 150 http://192.168.60.159 \
      | tee /dev/stderr)"
    [[ "${result}" == 'hello-world' ]]
  )
}

main "$@"
