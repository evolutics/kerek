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
      kerek tunnel-ssh --local-port 22375 --ssh-config ssh_config ssh-host &
      trap 'kill %%' EXIT
      until docker --host tcp://localhost:22375 ps; do
        sleep 0.01s
      done

      docker compose config --images \
        | kerek --host tcp://localhost:22375 transfer-images -

      kerek --host tcp://localhost:22375 \
        deploy --no-build --pull never --remove-orphans --wait
    )

    local -r result="$(curl --fail-with-body --max-time 3 --retry 99 \
      --retry-connrefused --retry-max-time 150 http://192.168.60.159)"
    if [[ "${result}" != 'hello-world' ]]; then
      echo "Unexpected result of sanity check: ${result}" >&2
      exit 1
    fi
  )
}

main "$@"
