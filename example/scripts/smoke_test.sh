#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo "Smoke testing" >>log.txt

curl --connect-timeout 3 --fail --retry 2 --show-error http://"$1":8080

if [[ "$1" == "192.168.62.62" ]]; then
  exit 1
fi
