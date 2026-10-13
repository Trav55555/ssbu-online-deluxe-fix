import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

import runtime_preflight


class RuntimePreflightTests(unittest.TestCase):
    EXPECTED_ROLES = (
        "plugin", "skyline_subsdk", "skyline_npdm", "ssbusync", "nx_over",
        "pia_manager", "imgui_smash", "smashline", "nro_hook", "arcropolis",
    )
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ssbu-preflight-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        (self.repo / "SSBU_VERSION").write_text("13.0.5\n")
        self.profile = self.root / "profile"
        self.profile.mkdir()
        self.emulator = self.root / "fake-emulator"
        self.emulator.write_text("#!/bin/sh\nexit 99\n")
        self.emulator.chmod(0o700)
        self.game = self.root / "fake-game"
        self.game.write_bytes(b"fixture only, no proprietary game data")
        binary = bytearray(256)
        binary[16:20] = b"NRO0"
        struct.pack_into("<I", binary, 24, len(binary))
        marker = b"13.0.5\0Unsupported SSBU version!\0DELUXE_EVENT \0"
        binary[128:128 + len(marker)] = marker
        self.config = {
            "schema": 1, "game_version": "13.0.5", "isolated_profile": True,
            "profile": str(self.profile), "emulator": str(self.emulator),
            "game": str(self.game), "files": {},
        }
        for role in self.EXPECTED_ROLES:
            path = self.profile / role
            path.write_bytes(binary)
            self.config["files"][role] = {
                "path": str(path), "sha256": hashlib.sha256(binary).hexdigest(),
            }
        self.path = self.root / "config.json"

    def inspect(self):
        self.path.write_text(json.dumps(self.config))
        return runtime_preflight.preflight(self.path, self.repo)

    def test_complete_configuration_does_not_certify_runtime(self):
        checks, observed = self.inspect()
        self.assertTrue(checks)
        self.assertTrue(all(c["status"] == "passed" for c in checks), checks)
        self.assertEqual(observed["declared_game_version"], "13.0.5")
        self.assertEqual(set(observed["files"]), set(self.EXPECTED_ROLES))

    def test_missing_dependency_or_unknown_checksum_blocks(self):
        for change in ("missing", "checksum"):
            with self.subTest(change=change):
                entry = self.config["files"]["pia_manager"]
                if change == "missing":
                    entry["path"] = str(self.profile / "missing")
                else:
                    entry["path"] = str(self.profile / "pia_manager")
                    entry["sha256"] = "not recorded"
                checks, _ = self.inspect()
                self.assertIn("blocked", [c["status"] for c in checks if c["name"] == "pia_manager"])

    def test_wrong_declared_version_blocks(self):
        self.config["game_version"] = "13.0.4"
        checks, _ = self.inspect()
        self.assertIn("blocked", [c["status"] for c in checks if c["name"] == "declared_game_version"])

    def test_checksum_mismatch_fails(self):
        self.config["files"]["plugin"]["sha256"] = "0" * 64
        checks, _ = self.inspect()
        self.assertIn("failed", [c["status"] for c in checks if c["name"] == "plugin"])

    def test_legacy_or_non_diagnostic_plugin_cannot_pass(self):
        plugin = self.profile / "plugin"
        original = plugin.read_bytes()
        for old, new, expected in [
            (b"13.0.5", b"13.0.4", "failed"),
            (b"DELUXE_EVENT ", b"no_trace_here", "blocked"),
        ]:
            with self.subTest(expected=expected):
                data = original.replace(old, new)
                plugin.write_bytes(data)
                self.config["files"]["plugin"]["sha256"] = hashlib.sha256(data).hexdigest()
                checks, _ = self.inspect()
                self.assertIn(expected, [c["status"] for c in checks if c["name"] == "plugin"])

    def test_profile_escape_and_symlink_escape_fail(self):
        outside = self.root / "outside.nro"
        outside.write_bytes((self.profile / "plugin").read_bytes())
        for path in (outside, self.profile / "escaped.nro"):
            if path != outside:
                path.symlink_to(outside)
            self.config["files"]["plugin"]["path"] = str(path)
            checks, _ = self.inspect()
            self.assertIn("failed", [c["status"] for c in checks if c["name"] == "plugin"])

    def test_default_profile_is_rejected_without_writing(self):
        self.config["profile"] = str(Path.home() / ".local/share/eden")
        checks, _ = self.inspect()
        self.assertIn("failed", [c["status"] for c in checks if c["name"] == "profile"])

    def run_cli(self, output):
        self.path.write_text(json.dumps(self.config))
        return subprocess.run([
            sys.executable, str(Path(runtime_preflight.__file__).resolve()),
            "--config", str(self.path), "--output-dir", str(output),
        ], capture_output=True, text=True)

    def test_cli_status_receipts_preservation_and_no_launch(self):
        before = {p.name: p.read_bytes() for p in self.profile.iterdir()}
        output = self.root / "evidence"
        result = self.run_cli(output)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads((output / "results.json").read_text())
        self.assertEqual(report["status"], "passed")
        self.assertEqual(report["scope"], "preflight")
        self.assertEqual(report["runtime"]["status"], "blocked")
        self.assertEqual((output / "events.jsonl").read_text(), "")
        existing = {p.name: p.read_bytes() for p in output.iterdir()}
        self.assertEqual(self.run_cli(output).returncode, 1)
        self.assertEqual(existing, {p.name: p.read_bytes() for p in output.iterdir()})
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.profile.iterdir()})

    def test_cli_reports_blocked_and_failure_as_nonzero(self):
        self.config["game_version"] = "13.0.4"
        blocked = self.root / "blocked"
        self.assertEqual(self.run_cli(blocked).returncode, 2)
        self.assertEqual(json.loads((blocked / "results.json").read_text())["status"], "blocked")
        self.config["files"]["plugin"]["sha256"] = "0" * 64
        failed = self.root / "failed"
        self.assertEqual(self.run_cli(failed).returncode, 1)
        self.assertEqual(json.loads((failed / "results.json").read_text())["status"], "failed")

    def test_cli_missing_prerequisite_cannot_pass(self):
        del self.config["files"]["pia_manager"]
        output = self.root / "missing-prerequisite"
        self.assertEqual(self.run_cli(output).returncode, 2)
        self.assertEqual(json.loads((output / "results.json").read_text())["status"], "blocked")

    def test_cli_never_writes_evidence_in_profile(self):
        output = self.profile / "evidence"
        self.assertEqual(self.run_cli(output).returncode, 1)
        self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
