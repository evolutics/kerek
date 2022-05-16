#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo 'Acceptance testing' >>log.txt

result="$(curl --data 'Boo Far' --fail --show-error http://"${KEREK_IP}":8080)"
readonly result
if [[ "${result}" != *'Boo Far'* ]]; then
  exit 1
fi
