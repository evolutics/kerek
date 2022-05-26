#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

do_package_management_setup() {
  sudo apt-get update
  sudo apt-get --yes upgrade
  sudo apt-get install unattended-upgrades
}

test_package_management_setup() {
  [[ ! -f /var/run/reboot-required ]]

  systemctl is-active unattended-upgrades.service
}

do_kubernetes_setup() {
  curl --fail --location --silent https://get.k3s.io | sh -
}

test_kubernetes_setup() {
  k3s check-config
}

do_data_folder_setup() {
  sudo mkdir /data
}

test_data_folder_setup() {
  [[ -d /data ]]
}

do_user_setup() {
  sudo sed --in-place 's/^#\(PermitRootLogin\) .*$/\1 no/' /etc/ssh/sshd_config

  sudo useradd --create-home --user-group kerek
  sudo rsync --archive --chown kerek:kerek "${HOME}/.ssh" /home/kerek

  echo '%kerek ALL=(ALL:ALL) NOPASSWD:ALL' \
    | sudo EDITOR='tee' visudo --file /etc/sudoers.d/kerek --strict
}

test_user_setup() {
  local -r sshd_configuration="$(sudo sshd -T)"
  echo "${sshd_configuration}" | grep '^passwordauthentication no$'
  echo "${sshd_configuration}" | grep '^permitrootlogin no$'

  diff <(groups kerek) <(echo 'kerek : kerek')
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
}

test_firewall_setup() {
  diff --ignore-trailing-space <(sudo ufw status verbose) <(echo 'Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), deny (routed)
New profiles: skip

To                         Action      From
--                         ------      ----
80/tcp                     ALLOW IN    Anywhere
443                        ALLOW IN    Anywhere
22/tcp                     ALLOW IN    Anywhere
6443                       ALLOW IN    Anywhere
80/tcp (v6)                ALLOW IN    Anywhere (v6)
443 (v6)                   ALLOW IN    Anywhere (v6)
22/tcp (v6)                ALLOW IN    Anywhere (v6)
6443 (v6)                  ALLOW IN    Anywhere (v6)
')
}

main() {
  for task in \
    package_management_setup \
    kubernetes_setup \
    data_folder_setup \
    user_setup \
    firewall_setup; do
    echo >&2 "● ${task}"
    "$1_${task}"
  done
}

main "$@"
