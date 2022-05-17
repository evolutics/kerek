#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

skaffold build --file-output .kerek/build.json
