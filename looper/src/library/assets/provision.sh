#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

set_up_k3s() {
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

main() {
  set_up_k3s
  set_up_data_folder
  set_up_deploy_user
}

main "$@"
