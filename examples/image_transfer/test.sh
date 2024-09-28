#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

main() {
  local -r box="unix://${PWD}/box.sock"
  CONTAINERS_STORAGE_CONF=storage.toml podman system service --time 0 "${box}" &
  trap 'fuser --kill -TERM box.sock' EXIT

  mapfile -t compresses < <(shuf --echo -- '' bzip2 gzip xz 'xz -9' zstd)

  for compress in "${compresses[@]}"; do
    echo "Compressing with: '${compress}'"

    podman --host "${box}" rmi --ignore docker.io/busybox
    [[ -z "$(podman --host "${box}" images --quiet)" ]]

    podman pull docker.io/busybox
    time kerek --container-engine podman --host "${box}" transfer-images \
      ${compress:+--compress "${compress}"} docker.io/busybox

    [[ "$(podman --host "${box}" images --format '{{.Repository}}:{{.Tag}}' \
      | tee /dev/stderr)" == 'docker.io/library/busybox:latest' ]]
  done
}

main "$@"
