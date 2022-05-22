#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo 'Base tests' >>log.txt

if grep 'Move to next version' log.txt; then
  exit 1
fi
