#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

main() {
  vagrant destroy --force
  trap 'vagrant destroy --force' EXIT
  vagrant up

  (
    trap 'rm --force ssh_config' EXIT
    vagrant ssh-config --host ssh-host >ssh_config

    "${KEREK}" provision --force --ssh-config ssh_config ssh-host

    docker compose pull --ignore-buildable

    docker compose config --images | "${KEREK}" run-with-ssh-config -- \
      ssh_config "${KEREK}" --host ssh://ssh-host transfer-images -

    "${KEREK}" run-with-ssh-config -- ssh_config "${KEREK}" \
      --host ssh://ssh-host deploy --no-build --pull never --remove-orphans \
      --wait

    local -r result="$(curl --fail-with-body --max-time 3 --retry 99 \
      --retry-connrefused --retry-max-time 150 http://192.168.60.159)"
    if [[ "${result}" != 'hello-world' ]]; then
      echo "Unexpected result of sanity check: ${result}" >&2
      exit 1
    fi
  )
}

main "$@"
