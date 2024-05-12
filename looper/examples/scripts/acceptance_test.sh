#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo 'Acceptance tests' >>log.txt

result="$(curl --fail-with-body http://"${KEREK_IP_ADDRESS}")"
readonly result
[[ "${result}" == *'hello-world'* ]]
