import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import package_release


class ReleasePackagingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.output = self.root / "dist"
        (self.root / "Cargo.toml").write_text('[package]\nversion = "1.1.0"\n')
        (self.root / "SSBU_VERSION").write_text("13.0.5\n")
        (self.root / "Cargo.lock").write_text(
            'version = 4\n[[package]]\nname = "ultelier"\nversion = "1.3.0"\n'
            'source = "git+https://example.test/ultelier#1234"\n'
        )
        for name in package_release.DOCUMENTS:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("Candidate documentation\n")
        (self.root / "release").mkdir()
        (self.root / "release/runtime-dependencies.json").write_text(
            '{"upstream_deluxe": {"tag": "v1.4.3"}}'
        )
        self.binary = bytearray(256)
        self.binary[16:20] = b"NRO0"
        struct.pack_into("<I", self.binary, 24, len(self.binary))
        marker = b"13.0.5\0Unsupported SSBU version!\0"
        self.binary[128:128 + len(marker)] = marker
        self.nro = self.root / package_release.BUILD_PATH
        self.nro.parent.mkdir(parents=True)
        self.nro.write_bytes(self.binary)
        git = patch.object(subprocess, "check_output", side_effect=["abc123\n", " M src/lib.rs\n"] * 10)
        git.start()
        self.addCleanup(git.stop)

    def package(self):
        return package_release.package_candidate(self.root, self.output)

    def test_package_has_correct_version_contents_and_checksums(self):
        archive = self.package()
        self.assertEqual(archive.name, "ssbu-online-deluxe-fix-v1.1.0-ssbu13.0.5.zip")
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        self.assertEqual(Path(str(archive) + ".sha256").read_text(), f"{digest}  {archive.name}\n")
        with zipfile.ZipFile(archive) as package:
            plugin = "atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_online_deluxe.nro"
            self.assertEqual(package.read(plugin), self.binary)
            self.assertEqual(set(package.namelist()), {
                plugin, "SSBU_VERSION", "Cargo.lock", "INSTALL.txt", "release.json",
                "README.md", "LICENSE", "THIRD_PARTY_NOTICES.md", "docs/version-support.md", "docs/testing.md",
            })
            manifest = json.loads(package.read("release.json"))
            self.assertEqual(manifest["ssbu_version"], "13.0.5")
            self.assertEqual(manifest["mod_version"], "1.1.0")
            self.assertEqual(manifest["status"], "candidate")
            self.assertTrue(manifest["source_dirty"])
            self.assertEqual(manifest["plugin_sha256"], hashlib.sha256(self.binary).hexdigest())
            self.assertEqual(manifest["runtime_dependencies"]["upstream_deluxe"]["tag"], "v1.4.3")
            self.assertIn(b"plugin-only", package.read("INSTALL.txt"))

    def test_rerun_is_identical_and_preserves_legacy_release(self):
        self.output.mkdir()
        legacy = self.output / "ssbu-online-deluxe-fix-v1.0.1.zip"
        legacy.write_bytes(b"unchanged legacy release")
        archive = self.package()
        original = archive.read_bytes()
        self.assertEqual(self.package().read_bytes(), original)
        self.assertEqual(legacy.read_bytes(), b"unchanged legacy release")

    def test_different_candidate_cannot_overwrite_existing_assets(self):
        archive = self.package()
        original = {p.name: p.read_bytes() for p in self.output.iterdir()}
        self.binary[200] = 1
        self.nro.write_bytes(self.binary)
        with self.assertRaisesRegex(FileExistsError, "Refusing to replace"):
            self.package()
        self.assertEqual({p.name: p.read_bytes() for p in self.output.iterdir()}, original)
        self.assertTrue(archive.exists())

    def test_wrong_version_binary_is_rejected_without_output(self):
        self.nro.write_bytes(bytes(self.binary).replace(b"13.0.5", b"13.0.4"))
        with self.assertRaisesRegex(ValueError, "game-version guard"):
            self.package()
        self.assertFalse(self.output.exists())

    def test_invalid_nro_and_truncated_nro_are_rejected(self):
        for binary, message in [(b"not an NRO", "not a Skyline NRO"), (self.binary[:-1], "NRO size")]:
            with self.subTest(message=message):
                self.nro.write_bytes(binary)
                with self.assertRaisesRegex(ValueError, message):
                    self.package()
        self.assertFalse(self.output.exists())

    def test_invalid_version_cannot_escape_output_directory(self):
        (self.root / "SSBU_VERSION").write_text("../../13.0.5")
        with self.assertRaisesRegex(ValueError, "Invalid SSBU version"):
            self.package()
        self.assertFalse(self.output.exists())

    def test_failed_build_cannot_create_release_assets(self):
        error = subprocess.CalledProcessError(101, "cargo skyline build")
        with patch.object(package_release.subprocess, "run", side_effect=error), \
             patch("sys.argv", ["package_release.py", "--output-dir", str(self.output)]):
            self.assertEqual(package_release.main(), 1)
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
