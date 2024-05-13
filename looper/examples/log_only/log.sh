#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

if [[ -v KEREK_ENVIRONMENT_ID ]]; then
  echo "$1: ${KEREK_ENVIRONMENT_ID}" >>log.txt
else
  echo "$1" >>log.txt
fi

if (("$(grep --count 'Move to next version' log.txt)" == 2)); then
  echo 'Break' >>log.txt
  exit 1
fi
