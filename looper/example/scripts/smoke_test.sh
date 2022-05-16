#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo 'Smoke testing' >>log.txt

curl --connect-timeout 3 --fail --retry 2 --show-error http://"${KEREK_IP}":8080
