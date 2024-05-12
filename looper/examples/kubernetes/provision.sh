#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo "Provision: ${KEREK_SSH_HOST}" >>log.txt

ssh -F "${KEREK_SSH_CONFIGURATION}" "${KEREK_SSH_HOST}" \
  KEREK_IP_ADDRESS="${KEREK_IP_ADDRESS}" 'bash -s' <<'EOF'
export INSTALL_K3S_EXEC="--disable traefik --tls-san ${KEREK_IP_ADDRESS}"
curl --fail --location --silent https://get.k3s.io | sh -

while ! sudo kubectl wait --all --for condition=Ready node; do
  sleep 1s
done
EOF

ssh -F "${KEREK_SSH_CONFIGURATION}" "${KEREK_SSH_HOST}" -- \
  sudo cat /etc/rancher/k3s/k3s.yaml >"${KUBECONFIG}"

kubectl config set-cluster default --server "https://${KEREK_IP_ADDRESS}:6443"

while ! kubectl get serviceaccount/default; do
  sleep 1s
done
