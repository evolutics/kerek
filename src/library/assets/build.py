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
        print(f"Removing obsolete file {str(obsolete_file)!r}.")
        obsolete_file.unlink()


def _build_image_file(build_context, images_folder):
    print(f"Building image for context {build_context!r}.")
    image_id = _build_image(build_context)

    image_file = images_folder / f"{image_id}.tar"

    if not image_file.exists():
        print(f"Saving image {image_id!r} to {str(image_file)!r}.")
        _save_image(image_id, image_file)

    return image_file


def _build_image(build_context):
    return subprocess.run(
        ["podman", "build", "--quiet", "--", build_context],
        capture_output=True,
        check=True,
        text=True,
    ).stdout.rstrip()


def _save_image(image_id, image_file):
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


if __name__ == "__main__":
    main()
