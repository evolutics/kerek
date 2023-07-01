#!/bin/bash
#
# `ssh` is a symbolic link to this file so that the code cleaner applies at all
# due to the filename extension.

set -o errexit
set -o nounset
set -o pipefail

/usr/bin/ssh -F ssh_configuration "$@"
