#!/usr/bin/env python3
"""Test production codec, match-state, and reporting logic without game FFI."""

from pathlib import Path
import subprocess
import sys
import tempfile


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    try:
        with tempfile.TemporaryDirectory(prefix="ssbu-host-tests-") as temp:
            for name in ("compatibility_host", "render_profile_host"):
                binary = Path(temp) / name
                subprocess.run(
                    ["rustc", "--edition=2021", "--test", str(root / "tests" / f"{name}.rs"),
                     "-o", str(binary)],
                    check=True, cwd=root,
                )
                subprocess.run([str(binary)], check=True, cwd=root)
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"Host regression tests failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
