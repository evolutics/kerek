#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo "Smoke tests: ${KEREK_SSH_HOST}" >>log.txt

curl --fail-with-body --max-time 3 --retry 99 --retry-connrefused \
  --retry-max-time 150 http://"${KEREK_IP_ADDRESS}"
