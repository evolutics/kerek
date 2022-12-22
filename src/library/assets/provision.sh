#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

do_package_management_setup() {
  sudo apt-get update
  sudo apt-get --yes upgrade
  sudo apt-get install --yes unattended-upgrades
}

test_package_management_setup() {
  [[ ! -f /var/run/reboot-required ]]

  systemctl is-active unattended-upgrades.service
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

do_podman_setup() {
  sudo apt-get install --yes podman
  sudo loginctl enable-linger kerek
}

test_podman_setup() {
  podman --version
  loginctl show-user kerek | grep '^Linger=yes$'
}

do_firewall_setup() {
  sudo apt-get install ufw
  sudo ufw --force reset

  sudo ufw default deny incoming

  sudo ufw allow http
  sudo ufw allow https
  sudo ufw allow ssh

  # TODO: Remove once example uses port 80 instead.
  sudo ufw allow 8080

  sudo ufw --force enable
}

test_firewall_setup() {
  diff --ignore-trailing-space <(sudo ufw status verbose) <(echo 'Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), disabled (routed)
New profiles: skip

To                         Action      From
--                         ------      ----
80/tcp                     ALLOW IN    Anywhere
443                        ALLOW IN    Anywhere
22/tcp                     ALLOW IN    Anywhere
8080                       ALLOW IN    Anywhere
80/tcp (v6)                ALLOW IN    Anywhere (v6)
443 (v6)                   ALLOW IN    Anywhere (v6)
22/tcp (v6)                ALLOW IN    Anywhere (v6)
8080 (v6)                  ALLOW IN    Anywhere (v6)
')
}

main() {
  for task in \
    package_management_setup \
    user_setup \
    podman_setup \
    firewall_setup; do
    echo >&2 "● ${task}"
    "$1_${task}"
  done
}

main "$@"
