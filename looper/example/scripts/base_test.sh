#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

echo 'Base testing' >>log.txt

if grep 'Moving to next version' log.txt; then
  exit 1
fi
