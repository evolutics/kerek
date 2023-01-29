#!/usr/bin/env python3

import os
import pathlib
import shlex
import subprocess


def main():
    subprocess.run(
        [
            "ansible-playbook",
            "--inventory",
            f",{os.environ['WHEELSTICKS_SSH_HOST']}",
            "--ssh-common-args",
            shlex.join(["-F", os.environ["WHEELSTICKS_SSH_CONFIGURATION"]]),
            "--",
            pathlib.Path(os.environ["WHEELSTICKS_PLAYBOOK"]),
        ],
        check=True,
    )


if __name__ == "__main__":
    main()
