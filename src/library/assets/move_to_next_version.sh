#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

while true; do
  git fetch --prune

  child_commit="$(git rev-list --ancestry-path --first-parent \
    HEAD..origin/main | tail -1)"

  if [[ -n "${child_commit}" ]]; then
    git checkout "${child_commit}"
    break
  fi

  sleep "$(("${RANDOM}" % 20))s"
done
