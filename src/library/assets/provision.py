#!/usr/bin/env python3

import datetime
import os
import pathlib
import subprocess
import time


def main():
    _do_provisioning()
    _reboot()
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
    _retry_subprocess(
        total_duration_limit=datetime.timedelta(seconds=150),
        retry_pause=datetime.timedelta(seconds=3),
        run=_try_to_test_provisioning,
    )


def _retry_subprocess(*, total_duration_limit, retry_pause, run):
    start = time.monotonic()

    while True:
        try:
            return run()
        except subprocess.SubprocessError:
            total_duration = datetime.timedelta(seconds=time.monotonic() - start)
            if total_duration >= total_duration_limit:
                raise

            time.sleep(retry_pause.total_seconds())


def _try_to_test_provisioning():
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
        timeout=datetime.timedelta(seconds=15).total_seconds(),
    )


if __name__ == "__main__":
    main()
