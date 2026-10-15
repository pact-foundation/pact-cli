#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""
Repackage the cargo-dist release archives as `pact-python-cli` wheels.

Each wheel holds the `pact` binary in its `.data/scripts/` directory, so
installers put it on `PATH`. The binary is copied byte-for-byte from the
release archive.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import io
import os
import tarfile
import time
import zipfile
from pathlib import Path

DIST_NAME = "pact_python_cli"
PROJECT_NAME = "pact-python-cli"

# The Windows ARM64 wheel carries the x86_64 binary, which runs under Windows'
# x64 emulation until `aarch64-pc-windows-msvc` is re-enabled in Cargo.toml.
TARGETS: dict[str, tuple[str, ...]] = {
    "aarch64-apple-darwin": ("macosx_11_0_arm64",),
    "x86_64-apple-darwin": ("macosx_11_0_x86_64",),
    "aarch64-unknown-linux-gnu": ("manylinux_2_34_aarch64",),
    "x86_64-unknown-linux-gnu": ("manylinux_2_34_x86_64",),
    "aarch64-unknown-linux-musl": ("musllinux_1_2_aarch64",),
    "x86_64-unknown-linux-musl": ("musllinux_1_2_x86_64",),
    "x86_64-pc-windows-msvc": ("win_amd64", "win_arm64"),
}

METADATA_HEADERS = [
    ("Metadata-Version", "2.4"),
    ("Name", PROJECT_NAME),
    ("Version", "{version}"),
    ("Summary", "Pact consolidated CLI: mock server, verifier, stub server, plugin CLI and broker client in a single binary"),
    ("Keywords", "contract-testing,pact,testing"),
    ("License-Expression", "MIT"),
    ("License-File", "LICENSE"),
    ("Project-URL", "Homepage, https://www.pact.io"),
    ("Project-URL", "Documentation, https://docs.pact.io"),
    ("Project-URL", "Source, https://github.com/pact-foundation/pact-cli"),
    ("Project-URL", "Issues, https://github.com/pact-foundation/pact-cli/issues"),
    ("Project-URL", "Changelog, https://github.com/pact-foundation/pact-cli/blob/main/CHANGELOG.md"),
    ("Classifier", "Development Status :: 5 - Production/Stable"),
    ("Classifier", "Environment :: Console"),
    ("Classifier", "Intended Audience :: Developers"),
    ("Classifier", "Operating System :: MacOS"),
    ("Classifier", "Operating System :: Microsoft :: Windows"),
    ("Classifier", "Operating System :: POSIX :: Linux"),
    ("Classifier", "Programming Language :: Rust"),
    ("Classifier", "Topic :: Software Development :: Testing"),
    ("Description-Content-Type", "text/markdown"),
]


def read_binary(archives: Path, target: str) -> tuple[str, bytes]:
    """Return the binary's file name and contents from a target's archive."""
    if "windows" in target:
        name = "pact.exe"
        with zipfile.ZipFile(archives / f"pact-{target}.zip") as zf:
            return name, zf.read(name)

    name = "pact"
    with tarfile.open(archives / f"pact-{target}.tar.xz") as tf:
        member = tf.extractfile(f"pact-{target}/{name}")
        if member is None:
            msg = f"pact-{target}/{name} is not a regular file"
            raise ValueError(msg)
        return name, member.read()


def record_hash(data: bytes) -> str:
    digest = hashlib.sha256(data).digest()
    return "sha256=" + base64.urlsafe_b64encode(digest).rstrip(b"=").decode()


def metadata(version: str, readme: str) -> bytes:
    headers = "".join(f"{key}: {value.format(version=version)}\n" for key, value in METADATA_HEADERS)
    return f"{headers}\n{readme}".encode()


def wheel_file(tag: str) -> bytes:
    return (
        "Wheel-Version: 1.0\n"
        "Generator: pact-cli build-wheels.py\n"
        "Root-Is-Purelib: false\n"
        f"Tag: py3-none-{tag}\n"
    ).encode()


def write_wheel(
    out: Path,
    version: str,
    tag: str,
    binary: tuple[str, bytes],
    readme: str,
    license_text: bytes,
    timestamp: tuple[int, int, int, int, int, int],
) -> Path:
    dist_info = f"{DIST_NAME}-{version}.dist-info"
    binary_name, binary_data = binary
    files: list[tuple[str, bytes, int]] = [
        (f"{DIST_NAME}-{version}.data/scripts/{binary_name}", binary_data, 0o755),
        (f"{dist_info}/METADATA", metadata(version, readme), 0o644),
        (f"{dist_info}/WHEEL", wheel_file(tag), 0o644),
        (f"{dist_info}/licenses/LICENSE", license_text, 0o644),
    ]

    record = io.StringIO()
    for path, data, _ in files:
        record.write(f"{path},{record_hash(data)},{len(data)}\n")
    record.write(f"{dist_info}/RECORD,,\n")
    files.append((f"{dist_info}/RECORD", record.getvalue().encode(), 0o644))

    wheel_path = out / f"{DIST_NAME}-{version}-py3-none-{tag}.whl"
    with zipfile.ZipFile(wheel_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for path, data, mode in files:
            info = zipfile.ZipInfo(path, date_time=timestamp)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = (0o100000 | mode) << 16
            zf.writestr(info, data)
    return wheel_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, help="PEP 440 version, for example 1!0.10.10")
    parser.add_argument("--archives", required=True, type=Path, help="Directory of cargo-dist archives")
    parser.add_argument("--readme", required=True, type=Path)
    parser.add_argument("--license", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()

    readme = args.readme.read_text(encoding="utf-8")
    license_text = args.license.read_bytes()
    # Zip timestamps cannot predate 1980.
    epoch = max(int(os.environ.get("SOURCE_DATE_EPOCH", "315532800")), 315532800)
    timestamp = time.gmtime(epoch)[:6]

    args.out.mkdir(parents=True, exist_ok=True)
    for target, tags in TARGETS.items():
        binary = read_binary(args.archives, target)
        for tag in tags:
            path = write_wheel(args.out, args.version, tag, binary, readme, license_text, timestamp)
            print(path)


if __name__ == "__main__":
    main()
