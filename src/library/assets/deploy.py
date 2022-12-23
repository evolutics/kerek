#!/usr/bin/env python3

import os
import pathlib
import subprocess
import sys


def main():
    local_images_folder = pathlib.Path(os.environ["KEREK_CACHE_WORKBENCH"])
    remote_images_folder = pathlib.Path("/home") / "kerek" / "images"
    _synchronize_artifacts(local_images_folder, remote_images_folder)
    _deploy_on_remote(remote_images_folder)


def _synchronize_artifacts(local_images_folder, remote_images_folder):
    # TODO: Escape quotes.
    quoted_ssh_configuration = f"'{os.environ['KEREK_SSH_CONFIGURATION']}'"

    subprocess.run(
        [
            "rsync",
            "--archive",
            "--delete",
            "--rsh",
            f"ssh -F {quoted_ssh_configuration}",
            "--",
            f"{local_images_folder}/",
            f"kerek@{os.environ['KEREK_SSH_HOST']}:{remote_images_folder}",
        ],
        check=True,
    )


def _deploy_on_remote(remote_images_folder):
    subprocess.run(
        [
            "ssh",
            "-F",
            os.environ["KEREK_SSH_CONFIGURATION"],
            "-l",
            "kerek",
            os.environ["KEREK_SSH_HOST"],
            "--",
            "python3",
            "-",
            remote_images_folder,
        ],
        check=True,
        input=sys.argv[1],
        text=True,
    )


if __name__ == "__main__":
    main()
