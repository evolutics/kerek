#!/usr/bin/env python3

import datetime
import os
import pathlib
import subprocess
import time


def main():
    print("Doing provisioning.")
    _do_provisioning()
    print("Rebooting.")
    _reboot()
    print("Testing provisioning.")
    _test_provisioning()


def _do_provisioning():
    cache_folder = pathlib.Path(os.environ["KEREK_CACHE_FOLDER"])
    subprocess.run(
        [
            "ssh",
            "-F",
            os.environ["KEREK_SSH_CONFIGURATION"],
            os.environ["KEREK_SSH_HOST"],
            "--",
            "bash",
            "-s",
            "--",
            "do",
        ],
        check=True,
        input=(cache_folder / "provision_on_remote.sh").read_bytes(),
    )


def _reboot():
    subprocess.run(
        [
            "ssh",
            "-F",
            os.environ["KEREK_SSH_CONFIGURATION"],
            "-f",
            "-l",
            "kerek",
            os.environ["KEREK_SSH_HOST"],
            "--",
            "sudo",
            "reboot",
        ],
        check=True,
    )


def _test_provisioning():
    timeout = datetime.timedelta(seconds=5)

    while True:
        try:
            return _try_to_test_provisioning(timeout)
        except subprocess.SubprocessError:
            timeout *= 2
            time.sleep(datetime.timedelta(seconds=1).total_seconds())


def _try_to_test_provisioning(timeout):
    cache_folder = pathlib.Path(os.environ["KEREK_CACHE_FOLDER"])
    subprocess.run(
        [
            "ssh",
            "-F",
            os.environ["KEREK_SSH_CONFIGURATION"],
            "-l",
            "kerek",
            os.environ["KEREK_SSH_HOST"],
            "--",
            "bash",
            "-s",
            "--",
            "test",
        ],
        check=True,
        input=(cache_folder / "provision_on_remote.sh").read_bytes(),
        timeout=timeout.total_seconds(),
    )


if __name__ == "__main__":
    main()
