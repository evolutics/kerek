#!/bin/bash

set -o errexit -o nounset -o pipefail

main() {
  local -r box="unix://${PWD}/box.sock"
  CONTAINERS_STORAGE_CONF=storage.toml podman system service --time 0 "${box}" &
  trap 'fuser --kill -TERM box.sock' EXIT
  podman --url "${box}" system prune --all --force

  export CONTAINER_ENGINE=podman
  local -r image='localhost/kerek:latest'

  podman build --build-arg VERSION=0 --tag "${image}" .
  kerek --host "${box}" transfer-images -- "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'v0' ]]

  podman build --build-arg VERSION=1 --tag "${image}" .
  kerek --host "${box}" transfer-images -- "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'v0' ]]
  kerek --host "${box}" transfer-images --force -- "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'v1' ]]

  for compress in $(shuf --echo -- bzip2 gzip xz zstd); do
    podman build --build-arg "VERSION=${compress}" --tag "${image}" .
    kerek --host "${box}" transfer-images --force -- "${image}"
    [[ "$(podman --url "${box}" run --rm "${image}" \
      | tee /dev/stderr)" == "v${compress}" ]]
  done

  podman build --build-arg VERSION=2 --tag "${image}" .
  kerek --host "${box}" transfer-images --compress 'zstd --verbose' --force -- \
    "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'v2' ]]
}

main "$@"
