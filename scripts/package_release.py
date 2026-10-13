#!/usr/bin/env python3
"""Build and package one game-version-specific Deluxe plugin candidate."""

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tomllib
import zipfile

PLUGIN_PATH = "atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_online_deluxe.nro"
BUILD_PATH = "target/aarch64-skyline-switch/release/libssbu_online_deluxe.nro"
DOCUMENTS = ("README.md", "LICENSE", "THIRD_PARTY_NOTICES.md", "docs/version-support.md")


def package_candidate(root: Path, output: Path) -> Path:
    cargo = tomllib.loads((root / "Cargo.toml").read_text())
    version = cargo["package"]["version"]
    game_version = (root / "SSBU_VERSION").read_text().strip()
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?", version):
        raise ValueError("Invalid mod release version")
    if not re.fullmatch(r"\d+\.\d+\.\d+", game_version):
        raise ValueError("Invalid SSBU version")

    binary = (root / BUILD_PATH).read_bytes()
    if len(binary) < 0x80 or binary[0x10:0x14] != b"NRO0":
        raise ValueError("Build output is not a Skyline NRO")
    if struct.unpack_from("<I", binary, 0x18)[0] != len(binary):
        raise ValueError("NRO size does not match its header")
    if game_version.encode() not in binary or b"Unsupported SSBU version!" not in binary:
        raise ValueError("NRO does not contain the expected game-version guard")

    lock_bytes = (root / "Cargo.lock").read_bytes()
    lock = tomllib.loads(lock_bytes.decode())
    dependencies = json.loads((root / "release/runtime-dependencies.json").read_text())
    revision = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True
    ).strip()
    dirty = bool(subprocess.check_output(
        ["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=root, text=True
    ).strip())
    manifest = {
        "status": "candidate",
        "mod_version": version,
        "ssbu_version": game_version,
        "source_commit": revision,
        "source_dirty": dirty,
        "plugin_sha256": hashlib.sha256(binary).hexdigest(),
        "cargo_lock_sha256": hashlib.sha256(lock_bytes).hexdigest(),
        "git_dependencies": [
            {"name": p["name"], "version": p["version"], "source": p["source"]}
            for p in lock["package"] if p.get("source", "").startswith("git+")
        ],
        "runtime_dependencies": dependencies,
    }
    install = f"""SSBU Online Deluxe Fix v{version} candidate, SSBU {game_version} only

This is a plugin-only package. It is not a standalone installation.
1. Install the prerequisites listed in docs/version-support.md.
2. Remove conflicting latency-slider, vsync, or less-lag mods.
3. Copy this package's atmosphere folder to your SD root or emulator sdmc.
   Replace libssbu_online_deluxe.nro. Keep the prerequisite plugins installed.
4. Fully restart the Switch or emulator.

Do not use this build with another SSBU version. Use v1.0.1 for SSBU 13.0.4.
Official Quickplay/Elite behavior and 13.0.5 runtime stability are unverified.
Read docs/version-support.md for test and publication gates.
This package contains no game files, updates, DLC, keys, saves, or font assets.
"""
    entries = {name: (root / name).read_bytes() for name in DOCUMENTS}
    entries.update({
        PLUGIN_PATH: binary,
        "SSBU_VERSION": (game_version + "\n").encode(),
        "Cargo.lock": lock_bytes,
        "INSTALL.txt": install.encode(),
        "release.json": (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(),
    })
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(entries.items()):
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    zip_bytes = buffer.getvalue()
    filename = f"ssbu-online-deluxe-fix-v{version}-ssbu{game_version}.zip"
    files = {
        filename: zip_bytes,
        filename + ".sha256": (
            hashlib.sha256(zip_bytes).hexdigest() + "  " + filename + "\n"
        ).encode(),
    }
    output.mkdir(parents=True, exist_ok=True)
    for name, data in files.items():
        path = output / name
        if path.exists() and path.read_bytes() != data:
            raise FileExistsError(f"Refusing to replace a different release artifact: {path}")
    for name, data in files.items():
        path = output / name
        if not path.exists():
            with path.open("xb") as artifact:
                artifact.write(data)
    return output / filename


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain", default="skyline-v3")
    parser.add_argument("--output-dir", type=Path, default=Path("dist"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    try:
        rustup = shutil.which("rustup")
        if rustup is None:
            raise FileNotFoundError("rustup is required for the Skyline-patched toolchain")
        environment = os.environ.copy()
        environment["PATH"] = str(Path(rustup).parent) + os.pathsep + environment.get("PATH", "")
        environment["RUSTUP_TOOLCHAIN"] = args.toolchain
        subprocess.run(
            [rustup, "run", args.toolchain, "cargo", "skyline", "build", "--release", "--", "--locked"],
            cwd=root, env=environment, check=True,
        )
        output = args.output_dir if args.output_dir.is_absolute() else root / args.output_dir
        print(package_candidate(root, output))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Release packaging failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
