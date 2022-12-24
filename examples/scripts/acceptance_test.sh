#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo 'Acceptance tests' >>log.txt

result="$(curl --data 'Boo Far' --fail --show-error \
  http://"${KEREK_IP_ADDRESS}")"
readonly result
if [[ "${result}" != *'Boo Far'* ]]; then
  exit 1
fi
