#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo 'Base tests' >>log.txt

if [[ $(("$(grep --count 'Base tests' log.txt)" % 2)) == 0 ]]; then
  echo '---' >>log.txt
  exit 1
fi
