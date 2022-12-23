#!/usr/bin/env python3

import json
import os
import pathlib
import subprocess


def main():
    with pathlib.Path("images.json").open("br") as images_file:
        build_contexts = json.load(images_file)

    images_folder = pathlib.Path(os.environ["KEREK_CACHE_WORKBENCH"])

    image_files = {
        _build_image_file(build_context, images_folder)
        for build_context in build_contexts
    }

    obsolete_files = set(images_folder.iterdir()) - image_files
    for obsolete_file in sorted(obsolete_files):
        obsolete_file.unlink()


def _build_image_file(build_context, images_folder):
    image_id = subprocess.run(
        ["podman", "build", "--quiet", "--", build_context],
        capture_output=True,
        check=True,
        text=True,
    ).stdout.rstrip()

    image_file = images_folder / f"{image_id}.tar"

    if not image_file.exists():
        subprocess.run(
            [
                "podman",
                "save",
                "--format",
                "oci-archive",
                "--output",
                image_file,
                "--",
                image_id,
            ],
            check=True,
        )

    return image_file


if __name__ == "__main__":
    main()
