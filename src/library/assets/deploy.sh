#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

skaffold deploy --build-artifacts .kerek/build.json \
  --kubeconfig "${KEREK_KUBECONFIG}"
