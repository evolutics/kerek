#!/bin/bash
#
# Docker via SSH does not support SSH configuration files, hence this wrapper.

set -o errexit
set -o nounset
set -o pipefail

"${REAL_SSH}" -F "${KEREK_SSH_CONFIGURATION}" "$@"
