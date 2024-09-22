#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

if [[ "${CONTAINER_ENGINE}" != 'podman' ]]; then
  echo "Unsupported container engine: ${CONTAINER_ENGINE}"
  exit 1
fi

sudo apt-get --yes install podman

systemctl --user enable --now podman

sudo loginctl enable-linger

sudo sysctl --write net.ipv4.ip_unprivileged_port_start=80 \
  | sudo tee /etc/sysctl.d/99-custom.conf

podman system prune --all --filter until=720h --force
