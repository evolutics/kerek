#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

sudo apt-get --yes install podman

systemctl --user enable --now podman

sudo ln --force --symbolic /usr/bin/podman /usr/local/bin/docker

sudo sysctl --write net.ipv4.ip_unprivileged_port_start=80 \
  | sudo tee /etc/sysctl.d/99-custom.conf

podman system prune --all --filter until=720h --force
