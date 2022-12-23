#!/usr/bin/env python3

import csv
import dataclasses
import enum
import functools
import io
import json
import pathlib
import subprocess
import sys


def main():
    network = "main"

    target_image_ids = _load_target_images()
    images = _get_images()

    actual_images = {image for image in images if image.container_count != 0}
    target_images = {image for image in images if image.image_id in target_image_ids}

    changes = _plan_changes(actual_images, target_images)

    _create_network_if_not_exists(network)
    for change in changes:
        _apply_change(network, change)
    _collect_garbage()


@dataclasses.dataclass(frozen=True)
class _Image:
    container_count: int
    container_names: tuple[str, ...]
    digest: str
    image_id: str
    port_mappings: tuple[str, ...]


_Sign = enum.Enum("_Sign", ["ADD", "KEEP", "REMOVE"])


@dataclasses.dataclass
class _ContainerChange:
    container_name: str
    image_digest: str
    image_id: str
    port_mappings: tuple[str, ...]
    sign: _Sign


def _load_target_images():
    images_folder = pathlib.Path(sys.argv[1])
    image_files = sorted(images_folder.iterdir())
    for image_file in image_files:
        print(f"Loading image file {str(image_file)!r}.")
        subprocess.run(["podman", "load", "--input", image_file], check=True)
    return {image_file.stem for image_file in image_files}


def _get_images():
    images = json.loads(
        subprocess.run(
            ["podman", "images", "--format", "json"], check=True, stdout=subprocess.PIPE
        ).stdout
    )
    return {_parse_image_metadata(image) for image in images}


def _parse_image_metadata(image):
    labels = image["Labels"]
    return _Image(
        container_count=int(image["Containers"]),
        container_names=_csv_fields(labels.get("info.evolutics.kerek.container-names")),
        digest=image["Digest"],
        image_id=image["Id"],
        port_mappings=_csv_fields(labels.get("info.evolutics.kerek.port-mappings")),
    )


def _csv_fields(optional_string):
    string = "" if optional_string is None else optional_string
    records = csv.reader(io.StringIO(string))
    return tuple(field for record in records for field in record)


def _plan_changes(actual_images, target_images):
    changes = [
        _ContainerChange(
            container_name=container_name,
            image_digest=image.digest,
            image_id=image.image_id,
            port_mappings=image.port_mappings,
            sign=sign,
        )
        for sign, images in (
            (_Sign.REMOVE, actual_images),
            (_Sign.ADD, target_images),
        )
        for image in images
        for container_name in image.container_names
    ]

    # Reasons to order changes (stable sort):
    # 1. Zero-downtime deployments are possible if there are multiple replicas:
    #    while a container `x-0` is replaced, a load balancer can still forward
    #    traffic to a replica `x-1`; at any time, either replica is available.
    # 2. The following simplification is easier.
    # 3. Clear predictability.
    changes.sort(key=lambda change: change.container_name)

    def cancel_removal_followed_by_addition(previous_changes, next_change):
        if (
            previous_changes
            and previous_changes[-1].sign == _Sign.REMOVE
            and next_change.sign == _Sign.ADD
            and previous_changes[-1].container_name == next_change.container_name
            and previous_changes[-1].image_digest == next_change.image_digest
            # Other relevant fields are captured by comparing the image digest.
        ):
            base = previous_changes[-1]
            return previous_changes[:-1] + [dataclasses.replace(base, sign=_Sign.KEEP)]
        return previous_changes + [next_change]

    changes = functools.reduce(cancel_removal_followed_by_addition, changes, [])

    return changes


def _create_network_if_not_exists(network):
    try:
        subprocess.run(["podman", "network", "exists", "--", network], check=True)
    except subprocess.CalledProcessError as error:
        if error.returncode == 1:
            print(f"Creating network {network!r}.")
            subprocess.run(["podman", "network", "create", "--", network], check=True)
        else:
            raise


def _apply_change(network, change):
    # TODO: Restart containers on reboot with systemd.
    # TODO: Support volumes.

    operand = f"container {change.container_name!r} of image {change.image_digest!r}"

    summary, operation = {
        _Sign.ADD: (f"Adding {operand}.", _add_container),
        _Sign.KEEP: (f"Keeping {operand}.", lambda _: None),
        _Sign.REMOVE: (f"Removing {operand}.", _remove_container),
    }[change.sign]

    print(summary)
    operation(network, change)


def _add_container(network, change):
    subprocess.run(
        [
            "podman",
            "run",
            "--detach",
            "--name",
            change.container_name,
            "--network",
            network,
        ]
        + [f"--publish={port_mapping}" for port_mapping in change.port_mappings]
        + ["--restart", "always", "--", change.image_id],
        check=True,
    )
    # TODO: Wait until healthy if there is a health check.


def _remove_container(_network, change):
    subprocess.run(["podman", "stop", "--", change.container_name], check=True)
    subprocess.run(["podman", "rm", "--", change.container_name], check=True)


def _collect_garbage():
    print("Collecting garbage.")
    subprocess.run(["podman", "system", "prune", "--all", "--force"], check=True)


if __name__ == "__main__":
    main()
