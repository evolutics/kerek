#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo "Env tests: ${KEREK_SSH_HOST}" >>log.txt

result="$(curl --fail-with-body --max-time 3 --retry 99 --retry-connrefused \
  --retry-max-time 150 http://"${KEREK_IP_ADDRESS}")"
readonly result
[[ "${result}" == *'hello-world'* ]]

if (("$(grep --count 'Env tests:' log.txt)" == 3)); then
  echo '---' >>log.txt
  exit 1
fi
