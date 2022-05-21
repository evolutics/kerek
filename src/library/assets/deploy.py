#!/usr/bin/env python3

import json
import os
import pathlib
import subprocess


def main():
    build_file = pathlib.Path(".kerek") / "build.json"
    local_images_file = _save_images(build_file)
    _import_images(local_images_file)
    _deploy_artifacts(build_file)


def _save_images(build_file):
    with build_file.open() as build:
        build = json.load(build)
    images = [artifact["tag"] for artifact in build["builds"]]

    local_images_file = pathlib.Path(".kerek") / "images.tar"
    subprocess.run(
        ["docker", "save", "--output", local_images_file, "--"] + images,
        check=True,
    )

    return local_images_file


def _import_images(local_images_file):
    remote_images_file = pathlib.Path("/tmp") / "images.tar"

    subprocess.run(
        [
            "scp",
            "-F",
            os.getenv("KEREK_SSH_CONFIGURATION"),
            "--",
            local_images_file,
            f"{os.getenv('KEREK_SSH_HOST')}:{remote_images_file}",
        ],
        check=True,
    )

    subprocess.run(
        [
            "ssh",
            "-F",
            os.getenv("KEREK_SSH_CONFIGURATION"),
            os.getenv("KEREK_SSH_HOST"),
            "--",
            "bash",
            "-s",
            "--",
            remote_images_file,
        ],
        check=True,
        input="""#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

sudo k3s ctr images import "$1"
rm "$1"
""",
        text=True,
    )


def _deploy_artifacts(build_file):
    subprocess.run(
        [
            "skaffold",
            "deploy",
            "--build-artifacts",
            build_file,
            "--kubeconfig",
            os.getenv("KEREK_KUBECONFIG"),
        ],
        check=True,
    )


if __name__ == "__main__":
    main()
