#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

update_package_index() {
  sudo apt-get update
}

set_up_automatic_upgrades() {
  sudo apt-get install unattended-upgrades
  systemctl is-active unattended-upgrades.service
}

set_up_kubernetes() {
  curl --fail --location --silent https://get.k3s.io | sh -
}

set_up_data_folder() {
  sudo mkdir /data
}

set_up_deploy_user() {
  sudo useradd --create-home --user-group deploy
  sudo rsync --archive --chown deploy:deploy "${HOME}/.ssh" /home/deploy

  echo "%deploy ALL=NOPASSWD: \
/usr/local/bin/k3s ctr images import /home/deploy/images.tar" \
    | sudo EDITOR='tee' visudo --file /etc/sudoers.d/deploy --strict
}

set_up_firewall() {
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

main() {
  update_package_index
  set_up_automatic_upgrades
  set_up_kubernetes
  set_up_data_folder
  set_up_deploy_user
  set_up_firewall
}

main "$@"
