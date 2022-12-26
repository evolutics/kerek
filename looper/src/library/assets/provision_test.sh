#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

set -o xtrace

test_package_management_setup() {
  [[ ! -f /var/run/reboot-required ]]

  systemctl is-active unattended-upgrades.service
}

test_user_setup() {
  local -r sshd_configuration="$(sudo sshd -T)"
  echo "${sshd_configuration}" | grep '^passwordauthentication no$'
  echo "${sshd_configuration}" | grep '^permitrootlogin no$'

  [[ "$(groups kerek)" == 'kerek : kerek' ]]
}

test_podman_setup() {
  podman --version
  loginctl show-user kerek | grep '^Linger=yes$'
  [[ "$(sysctl --values net.ipv4.ip_unprivileged_port_start)" == 80 ]]
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
80/tcp (v6)                ALLOW IN    Anywhere (v6)
443 (v6)                   ALLOW IN    Anywhere (v6)
22/tcp (v6)                ALLOW IN    Anywhere (v6)
')
}

main() {
  test_package_management_setup
  test_user_setup
  test_podman_setup
  test_firewall_setup
}

main "$@"
