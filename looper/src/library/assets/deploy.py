#!/usr/bin/env python3

import os
import pathlib
import subprocess


def main():
    print("Synchronizing artifacts.")
    _synchronize_artifacts()
    print("Deploying on remote.")
    _deploy_on_remote()


def _synchronize_artifacts():
    # TODO: Escape quotes.
    quoted_ssh_configuration = f"'{os.environ['KEREK_SSH_CONFIGURATION']}'"
    destination = f"kerek@{os.environ['KEREK_SSH_HOST']}"

    subprocess.run(
        [
            "rsync",
            "--archive",
            "--delete",
            "--rsh",
            f"ssh -F {quoted_ssh_configuration}",
            "--",
            f"{os.environ['KEREK_CACHE_WORKBENCH']}/",
            f"{destination}:{os.environ['KEREK_REMOTE_IMAGES_FOLDER']}",
        ],
        check=True,
    )


def _deploy_on_remote():
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
            f"KEREK_REMOTE_IMAGES_FOLDER={os.environ['KEREK_REMOTE_IMAGES_FOLDER']}",
            "python3",
        ],
        check=True,
        input=(scripts_folder / "deploy_on_remote.py").read_bytes(),
    )


if __name__ == "__main__":
    main()
