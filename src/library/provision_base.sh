#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

curl --fail --location --silent https://get.k3s.io | sh -
