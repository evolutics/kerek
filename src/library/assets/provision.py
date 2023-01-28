#!/usr/bin/env python3

import os
import pathlib
import shlex
import subprocess


def main():
    scripts_folder = pathlib.Path(os.environ["KEREK_CACHE_SCRIPTS"])
    subprocess.run(
        [
            "ansible-playbook",
            "--inventory",
            f",{os.environ['KEREK_SSH_HOST']}",
            "--ssh-common-args",
            shlex.join(["-F", os.environ["KEREK_SSH_CONFIGURATION"]]),
            "--",
            scripts_folder / "playbook.yaml",
        ],
        check=True,
    )


if __name__ == "__main__":
    main()
