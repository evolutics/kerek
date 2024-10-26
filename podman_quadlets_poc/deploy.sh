#!/bin/bash

set -o errexit -o nounset -o pipefail

main() {
  vagrant ssh -- mkdir --parents .config/containers/systemd

  vagrant upload hi@.container .config/containers/systemd/
  vagrant upload reverse-proxy.container .config/containers/systemd/
  vagrant upload test-net.network .config/containers/systemd/

  if vagrant ssh --command \
    '[[ -e .config/containers/systemd/hi@A.container ]]'; then
    local -r add_version=B
    local -r remove_version=A
  else
    local -r add_version=A
    local -r remove_version=B
  fi

  vagrant ssh -- ln --symbolic hi@.container \
    ".config/containers/systemd/hi@${add_version}.container"
  vagrant ssh -- rm --force \
    ".config/containers/systemd/hi@${remove_version}.container"

  vagrant ssh -- systemctl --user daemon-reload

  vagrant ssh -- systemctl --user start "hi@${add_version}.service"
  vagrant ssh -- systemctl --user stop "hi@${remove_version}.service"
  vagrant ssh -- systemctl --user start reverse-proxy.service
}

main "$@"
