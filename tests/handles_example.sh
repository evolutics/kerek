#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

reset_vm() {
  vagrant destroy --force
  vagrant up
  vagrant ssh-config --host "${ssh_host}" >ssh_configuration
}

main() {
  export DOCKER_HOST="unix://${PWD}/podman.sock"
  export PATH="${PWD}/example/custom_bin:${PATH}"
  export WHEELSTICKS_VM_IP_ADDRESS='192.168.60.97'
  local -r ssh_host='example'

  cd example

  reset_vm
  git clean --force -X -- .wheelsticks

  podman system service --time 0 "${DOCKER_HOST}" &
  # shellcheck disable=SC2064
  trap "kill -SIGINT $!" EXIT
  sleep 2s

  docker compose build
  docker --host "${DOCKER_HOST}" save example-web \
    | docker --host "ssh://${ssh_host}" load

  "${WHEELSTICKS}" --host "ssh://${ssh_host}" deploy

  curl --fail --max-time 3 --retry 99 --retry-connrefused --retry-max-time 15 \
    --show-error "http://${WHEELSTICKS_VM_IP_ADDRESS}"
}

main "$@"
