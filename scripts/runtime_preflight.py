#!/usr/bin/env python3
"""Read-only checks for a dedicated Eden test profile. Does not launch a game."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys

ROLES = (
    "skyline_subsdk", "skyline_npdm", "ssbusync", "nx_over", "pia_manager",
    "imgui_smash", "smashline", "nro_hook", "arcropolis",
)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def inspect_nro(path):
    data = path.read_bytes()
    if len(data) < 128 or data[16:20] != b"NRO0":
        raise ValueError("not a Skyline NRO")
    if struct.unpack_from("<I", data, 24)[0] != len(data):
        raise ValueError("NRO header size does not match file size")
    return data


def preflight(config_path, root):
    checks = []
    observed = {}

    def check(name, status, detail):
        checks.append({"name": name, "status": status, "detail": detail})

    def file_path(value, name):
        if not isinstance(value, str) or not value:
            check(name, "blocked", "Configure an explicit file path")
            return None
        path = Path(value).expanduser()
        if not path.is_absolute():
            check(name, "failed", "Paths must be absolute")
            return None
        path = path.resolve()
        if not path.is_file():
            check(name, "blocked", "Required file is unavailable")
            return None
        return path

    try:
        config = json.loads(config_path.read_text())
    except FileNotFoundError:
        check("configuration", "blocked", "Configuration file is unavailable")
        return checks, observed
    except (OSError, ValueError) as error:
        check("configuration", "failed", str(error))
        return checks, observed
    if not isinstance(config, dict) or config.get("schema") != 1:
        check("configuration", "failed", "Expected configuration schema 1")
        return checks, observed

    required_version = (root / "SSBU_VERSION").read_text().strip()
    observed["declared_game_version"] = config.get("game_version")
    check("declared_game_version", "passed" if config.get("game_version") == required_version else "blocked",
          f"Requires {required_version}; declaration is not runtime evidence")
    emulator = file_path(config.get("emulator"), "emulator")
    if emulator:
        observed["emulator"] = {"path": str(emulator), "sha256": digest(emulator)}
        check("emulator", "passed" if os.access(emulator, os.X_OK) else "blocked", "Emulator must be executable; launch behavior is unverified")
    game = file_path(config.get("game"), "game")
    if game:
        observed["game"] = str(game)
        check("game", "passed", "Game file exists; ownership, contents, installed updates and effective version are unverified")

    profile_value = config.get("profile")
    if not isinstance(profile_value, str) or not Path(profile_value).is_absolute():
        check("profile", "failed", "Configure an absolute dedicated profile path")
        return checks, observed
    profile = Path(profile_value).expanduser().resolve()
    observed["profile"] = str(profile)
    home = Path.home().resolve()
    protected = [root.resolve(), home / ".local/share/eden", home / ".config/eden"]
    unsafe = profile == home or any(
        profile.is_relative_to(p) or p.is_relative_to(profile) for p in protected
    )
    if unsafe or config.get("isolated_profile") is not True:
        check("profile", "failed", "Use an explicitly isolated profile outside the repository and default Eden directories")
        return checks, observed
    if not profile.is_dir():
        check("profile", "blocked", "Dedicated profile directory is unavailable")
        return checks, observed
    observed["profile"] = str(profile)
    check("profile", "passed", "Paths are isolated; emulator launch configuration is not verified")

    files = config.get("files")
    if not isinstance(files, dict):
        check("files", "blocked", "Configure plugin and prerequisite file checksums")
        return checks, observed
    observed["files"] = {}
    for role in ("plugin", *ROLES):
        entry = files.get(role)
        if not isinstance(entry, dict):
            check(role, "blocked", "Required file/checksum is not configured")
            continue
        path = file_path(entry.get("path"), role)
        if path is None:
            continue
        if not path.is_relative_to(profile):
            check(role, "failed", "Runtime files must resolve inside the dedicated profile")
            continue
        expected = entry.get("sha256")
        if not isinstance(expected, str) or not re.fullmatch(r"[0-9a-f]{64}", expected):
            check(role, "blocked", "Configure an independently recorded SHA256")
            continue
        try:
            if path.stat().st_size > 64 * 1024 * 1024:
                raise ValueError("Runtime file exceeds the 64 MiB preflight limit")
            actual = digest(path)
            observed["files"][role] = {"path": str(path), "sha256": actual}
            if actual != expected:
                check(role, "failed", "SHA256 differs from configured expectation")
                continue
            if role not in ("skyline_subsdk", "skyline_npdm"):
                data = inspect_nro(path)
                if role == "plugin":
                    if required_version.encode() not in data or b"Unsupported SSBU version!" not in data:
                        check(role, "failed", "Required game-version guard is absent")
                        continue
                    if b"DELUXE_EVENT " not in data:
                        check(role, "blocked", "Build the plugin with --features runtime_trace")
                        continue
            check(role, "passed", "File matches configured checksum; runtime API/load compatibility is unverified")
        except (OSError, ValueError) as error:
            check(role, "failed", str(error))
    return checks, observed


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[1]
    try:
        checks, observed = preflight(args.config, root)
        status = "failed" if any(c["status"] == "failed" for c in checks) else (
            "blocked" if any(c["status"] == "blocked" for c in checks) else "passed"
        )
        revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
        dirty = bool(subprocess.check_output(
            ["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=root, text=True
        ).strip())
        # Never create receipts within the inspected profile or replace prior evidence.
        output = args.output_dir.resolve()
        for protected in (Path.home() / ".local/share/eden", Path.home() / ".config/eden"):
            protected = protected.resolve()
            if output.is_relative_to(protected) or protected.is_relative_to(output):
                raise ValueError("Cannot write evidence in the default Eden installation")
        if "profile" in observed:
            profile = Path(observed["profile"])
            if output.is_relative_to(profile) or profile.is_relative_to(output):
                raise ValueError("Evidence directory must be separate from the test profile")
        output.mkdir(parents=True, exist_ok=False)
        (output / "run.json").write_text(json.dumps({
            "schema": 1, "scope": "preflight", "source_commit": revision,
            "source_dirty": dirty, "config_sha256": digest(args.config) if args.config.is_file() else None,
            "observed": observed,
        }, indent=2) + "\n")
        (output / "results.json").write_text(json.dumps({
            "schema": 1, "scope": "preflight", "status": status, "checks": checks,
            "runtime": {"status": "blocked", "reason": "No game was launched or runtime evidence evaluated"},
        }, indent=2) + "\n")
        (output / "events.jsonl").write_text("")
        print(f"Preflight {status}: {output / 'results.json'}; runtime remains blocked")
        return {"passed": 0, "failed": 1, "blocked": 2}[status]
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Preflight failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
