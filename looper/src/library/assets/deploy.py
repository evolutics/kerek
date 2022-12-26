#!/usr/bin/env python3

import os
import pathlib
import subprocess


def main():
    local_images_folder = pathlib.Path(os.environ["KEREK_CACHE_WORKBENCH"])
    remote_images_folder = pathlib.Path("/home") / "kerek" / "images"
    print("Synchronizing artifacts.")
    _synchronize_artifacts(local_images_folder, remote_images_folder)
    print("Deploying on remote.")
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
    scripts_folder = pathlib.Path(os.environ["KEREK_CACHE_SCRIPTS"])
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
        input=(scripts_folder / "deploy_on_remote.py").read_bytes(),
    )


if __name__ == "__main__":
    main()
