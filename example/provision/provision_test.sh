#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

set -o xtrace

test_container_platform() {
  [[ "$(loginctl -P Linger show-user "${WHEELSTICKS_DEPLOY_USER}")" == "yes" ]]
  [[ "$(sysctl --values net.ipv4.ip_unprivileged_port_start)" == 80 ]]
}

test_firewall() {
  diff --ignore-trailing-space <(sudo ufw status verbose) <(echo 'Status: active
Logging: on (low)
Default: deny (incoming), allow (outgoing), disabled (routed)
New profiles: skip

To                         Action      From
--                         ------      ----
80/tcp                     ALLOW IN    Anywhere
443                        ALLOW IN    Anywhere
22/tcp                     ALLOW IN    Anywhere
80/tcp (v6)                ALLOW IN    Anywhere (v6)
443 (v6)                   ALLOW IN    Anywhere (v6)
22/tcp (v6)                ALLOW IN    Anywhere (v6)
')
}

test_package_management() {
  [[ ! -f /var/run/reboot-required ]]

  systemctl is-active unattended-upgrades.service
}

test_podman() {
  podman --version
  sudo systemctl --machine "${WHEELSTICKS_DEPLOY_USER}@" --user is-active podman.socket
  sudo systemctl --machine "${WHEELSTICKS_DEPLOY_USER}@" --user is-enabled podman.socket
  docker --version
}

test_user_management() {
  sudo sshd -T | grep '^passwordauthentication no$'
  sudo sshd -T | grep '^permitrootlogin no$'

  [[ "$(groups "${WHEELSTICKS_DEPLOY_USER}")" == "${WHEELSTICKS_DEPLOY_USER} : ${WHEELSTICKS_DEPLOY_USER}" ]]
  ! sudo --user "${WHEELSTICKS_DEPLOY_USER}" -- sudo --non-interactive --validate
}

main() {
  test_container_platform
  test_firewall
  test_package_management
  test_podman
  test_user_management
}

main "$@"
