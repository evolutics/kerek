#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

main() {
  local -r box="unix://${PWD}/box.sock"
  CONTAINERS_STORAGE_CONF=storage.toml podman system service --time 0 "${box}" &
  trap 'fuser --kill -TERM box.sock' EXIT
  podman --url "${box}" system prune --all --force

  export CONTAINER_ENGINE=podman
  local -r image='localhost/kerek:latest'

  podman build --build-arg VERSION=A --tag "${image}" .
  kerek --host "${box}" transfer-images -- "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'vA' ]]

  podman build --build-arg VERSION=B --tag "${image}" .
  kerek --host "${box}" transfer-images -- "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'vA' ]]
  kerek --host "${box}" transfer-images --force -- "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'vB' ]]

  for compress in $(shuf --echo -- bzip2 gzip xz zstd); do
    podman build --build-arg "VERSION=${compress}" --tag "${image}" .
    kerek --host "${box}" transfer-images --force -- "${image}"
    [[ "$(podman --url "${box}" run --rm "${image}" \
      | tee /dev/stderr)" == "v${compress}" ]]
  done

  podman build --build-arg VERSION=C --tag "${image}" .
  kerek --host "${box}" transfer-images --compress 'zstd --verbose' --force -- \
    "${image}"
  [[ "$(podman --url "${box}" run --rm "${image}" | tee /dev/stderr)" == 'vC' ]]
}

main "$@"
