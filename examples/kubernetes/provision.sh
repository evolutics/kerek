#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

ssh -F "${KEREK_SSH_CONFIGURATION}" "${KEREK_SSH_HOST}" -- '
curl --fail --location --silent https://get.k3s.io | sh -s - --disable traefik
sudo kubectl wait --all --for condition=Ready --timeout 60s node'

ssh -F "${KEREK_SSH_CONFIGURATION}" "${KEREK_SSH_HOST}" -- \
  sudo cat /etc/rancher/k3s/k3s.yaml >"${KUBECONFIG}"

kubectl config set-cluster default --server "https://${KEREK_IP_ADDRESS}:6443"
