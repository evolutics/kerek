#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

do_package_index_update() {
  sudo apt-get update
}

test_package_index_update() {
  true
}

do_automatic_upgrades_setup() {
  sudo apt-get install unattended-upgrades
}

test_automatic_upgrades_setup() {
  systemctl is-active unattended-upgrades.service
}

do_kubernetes_setup() {
  curl --fail --location --silent https://get.k3s.io | sh -
}

test_kubernetes_setup() {
  true
}

do_data_folder_setup() {
  sudo mkdir /data
}

test_data_folder_setup() {
  true
}

do_deploy_user_setup() {
  sudo useradd --create-home --user-group deploy
  sudo rsync --archive --chown deploy:deploy "${HOME}/.ssh" /home/deploy

  echo "%deploy ALL=NOPASSWD: \
/usr/local/bin/k3s ctr images import /home/deploy/images.tar" \
    | sudo EDITOR='tee' visudo --file /etc/sudoers.d/deploy --strict
}

test_deploy_user_setup() {
  true
}

do_firewall_setup() {
  sudo apt-get install ufw
  sudo ufw --force reset

  sudo ufw default deny incoming

  sudo ufw allow http
  sudo ufw allow https
  sudo ufw allow ssh

  local -r KUBERNETES_API_SERVER_PORT=6443
  sudo ufw allow "${KUBERNETES_API_SERVER_PORT}"

  sudo ufw --force enable
  sudo ufw status verbose
}

test_firewall_setup() {
  true
}

main() {
  for task in \
    package_index_update \
    automatic_upgrades_setup \
    kubernetes_setup \
    data_folder_setup \
    deploy_user_setup \
    firewall_setup; do
    "$1_${task}"
  done
}

main "$@"
