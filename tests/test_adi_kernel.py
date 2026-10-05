"""Fast offline contract/security tests; real kernel builds are separate."""
import importlib.util
import io
import json
import multiprocessing
from pathlib import Path
import shutil
import struct
import subprocess
import tarfile
import tempfile
import time
import unittest
from unittest.mock import patch

HELPER = Path(__file__).resolve().parents[1] / "targets/adi-linux/build-kernel.py"
spec = importlib.util.spec_from_file_location("kernel", HELPER)
assert spec is not None and spec.loader is not None
kernel = importlib.util.module_from_spec(spec)
spec.loader.exec_module(kernel)


def lock_worker(path, queue):
    with kernel.lock(path):
        queue.put("entered")


class KernelTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def payload(self, name):
        data = bytearray(128)
        if name == "zynq":
            data[36:40] = b"\x18\x28\x6f\x01"
        else:
            data[56:60] = b"ARM\x64"
        return bytes(data)

    def artifact(self, name):
        generation = self.root / "image-test"
        generation.mkdir(exist_ok=True)
        path = generation / kernel.TARGETS[name]["output"]
        payload = self.payload(name)
        path.write_bytes(kernel.uimage(payload) if name == "zynq" else payload)
        data = {"schema_version": 1, "platform": name, "kernel_image": str(path),
                "sha256": kernel.digest(path), "provenance": kernel.provenance(name)}
        manifest = self.root / "artifacts.json"
        manifest.write_text(json.dumps(data))
        return manifest, path, data

    def test_git_targets_are_self_contained(self):
        for target in HELPER.parents[1].glob("adi-linux-*-*/"):
            self.assertEqual((target / "build-kernel.py").read_bytes(), HELPER.read_bytes())
            manifest = (target / "sdk.yml").read_text()
            self.assertIn("source: build-kernel.py", manifest)
            self.assertNotIn("../", manifest)

    def test_release_selection_and_cache_isolation(self):
        self.assertEqual(kernel.provenance("zynq"), kernel.provenance("zynq", "2023_R2"))
        for name in kernel.TARGETS:
            manifest, image, data = self.artifact(name)
            with self.assertRaisesRegex(ValueError, "provenance"):
                kernel.validate_manifest(manifest, name, "2026_R1")
            data["provenance"] = kernel.provenance(name, "2026_R1")
            manifest.write_text(json.dumps(data))
            result = subprocess.run(["python3", HELPER, "--platform", name,
                                     "--release", "2026_R1", "--output", self.root,
                                     "--verify"], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(data["provenance"]["source"]["ref"], "xlnx_2026.1.0")
            self.assertEqual(data["provenance"]["source"]["ref_type"], "tag")
            with patch.object(kernel, "download", side_effect=RuntimeError("must not download")):
                with self.assertRaisesRegex(ValueError, "provenance"):
                    kernel.build(name, self.root, self.root / "cache", 1)
        self.assertIn(b"ADI Linux 2026_R1", kernel.uimage(self.payload("zynq"), "2026_R1")[:64])

    def test_new_target_recipes_select_release(self):
        for name in kernel.TARGETS:
            text = (HELPER.parents[1] / f"adi-linux-2026-r1-{name}/sdk.yml").read_text()
            self.assertIn("--release 2026_R1", text)
            self.assertIn(f"artifacts/2026_R1/{name}", text)
        result = subprocess.run(["python3", HELPER, "--platform", "zynq", "--release",
                                 "xlnx_2026.1.0", "--output", self.root], capture_output=True)
        self.assertNotEqual(result.returncode, 0)

    def test_both_contracts(self):
        for name in kernel.TARGETS:
            manifest, _, data = self.artifact(name)
            self.assertEqual(kernel.validate_manifest(manifest, name), data)
            result = subprocess.run(["python3", HELPER, "--platform", name,
                                     "--output", self.root, "--verify"], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_corrupt_artifact(self):
        manifest, image, _ = self.artifact("zynq")
        image.write_bytes(b"bad")
        with self.assertRaisesRegex(ValueError, "checksum"):
            kernel.validate_manifest(manifest, "zynq")

    def test_provenance_and_schema_fail_closed(self):
        for field, value in [("schema_version", 2), ("platform", "zynqmp"), ("provenance", {})]:
            manifest, _, data = self.artifact("zynq")
            data[field] = value
            manifest.write_text(json.dumps(data))
            with self.assertRaises(ValueError):
                kernel.validate_manifest(manifest, "zynq")

    def test_escape_and_symlink_rejected(self):
        manifest, image, data = self.artifact("zynq")
        outside = self.root / "outside"
        shutil.copy(image, outside)
        image.unlink()
        image.symlink_to(outside)
        with self.assertRaisesRegex(ValueError, "Unsafe"):
            kernel.validate_manifest(manifest, "zynq")
        data["kernel_image"] = str(outside)
        manifest.write_text(json.dumps(data))
        with self.assertRaises(ValueError):
            kernel.validate_manifest(manifest, "zynq")

    def test_cache_rehash_and_failed_download_not_published(self):
        source = self.root / "source"
        source.write_bytes(b"verified")
        spec = {"url": source.as_uri(), "sha256": kernel.digest(source)}
        cache = self.root / "cache"
        downloaded = kernel.download(spec, cache)
        source.unlink()
        self.assertEqual(kernel.download(spec, cache), downloaded)
        downloaded.write_bytes(b"corrupt")
        with self.assertRaisesRegex(ValueError, "Corrupt"):
            kernel.download(spec, cache)
        downloaded.unlink()
        source.write_bytes(b"wrong")
        with self.assertRaisesRegex(ValueError, "mismatch"):
            kernel.download(spec, cache)
        self.assertFalse(downloaded.exists())
        self.assertFalse(list(cache.glob(".download-*")))

    def test_tar_traversal_and_escaping_symlink(self):
        for kind in ("path", "symlink"):
            archive = self.root / "bad.tar"
            with tarfile.open(archive, "w") as tar:
                entry = tarfile.TarInfo("../escape" if kind == "path" else "link")
                if kind == "symlink":
                    entry.type = tarfile.SYMTYPE
                    entry.linkname = "/etc/passwd"
                tar.addfile(entry, io.BytesIO())
            with self.assertRaises(tarfile.FilterError):
                kernel.extract(archive, self.root / "extract")
        self.assertFalse((self.root / "escape").exists())

    def test_uimage_independent_header_and_mkimage(self):
        _, image, _ = self.artifact("zynq")
        header = struct.unpack(">7I4B32s", image.read_bytes()[:64])
        self.assertEqual(header[4:6], (0x8000, 0x8000))
        self.assertEqual(header[7:11], (5, 2, 2, 0))
        if shutil.which("mkimage"):
            result = subprocess.run(["mkimage", "-l", image], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("00008000", result.stdout)
        image.write_bytes(image.read_bytes()[:-1] + b"x")
        with self.assertRaisesRegex(ValueError, "CRC"):
            kernel.validate_image(image, "zynq")

    def test_lock_serializes_processes(self):
        context = multiprocessing.get_context("spawn")
        queue = context.Queue()
        path = self.root / "lock"
        with kernel.lock(path):
            process = context.Process(target=lock_worker, args=(path, queue))
            process.start()
            time.sleep(0.1)
            self.assertTrue(queue.empty())
        self.assertEqual(queue.get(timeout=5), "entered")
        process.join(timeout=5)
        self.assertEqual(process.exitcode, 0)

    def test_cache_hit_does_not_build_and_failed_force_preserves_old(self):
        manifest, image, _ = self.artifact("zynq")
        before = manifest.read_bytes(), image.read_bytes()
        with patch.object(kernel, "download", side_effect=RuntimeError("offline")):
            self.assertEqual(kernel.build("zynq", self.root, self.root / "cache", 1), manifest)
            with self.assertRaises(RuntimeError):
                kernel.build("zynq", self.root, self.root / "cache", 1, force=True)
        self.assertEqual((manifest.read_bytes(), image.read_bytes()), before)

    def test_build_commands_publication_and_generation_replacement(self):
        for name in kernel.TARGETS:
            output = self.root / name
            commands = []

            def extract(_archive, destination):
                if destination.name == "source":
                    (destination / ("linux-" + kernel.COMMIT)).mkdir(parents=True)
                else:
                    compiler = destination / "bin" / (kernel.TARGETS[name]["triple"] + "-gcc")
                    compiler.parent.mkdir(parents=True)
                    compiler.touch()

            def run(command, env):
                commands.append(command)
                build = Path(next(x[2:] for x in command if isinstance(x, str) and x.startswith("O=")))
                image = build / "arch" / kernel.TARGETS[name]["arch"] / "boot" / kernel.TARGETS[name]["image"]
                image.parent.mkdir(parents=True, exist_ok=True)
                image.write_bytes(self.payload(name))
                self.assertNotIn("MAKEFLAGS", env)

            with patch.object(kernel, "download", return_value=self.root / "archive"), patch.object(kernel, "extract", side_effect=extract), patch.object(kernel, "run", side_effect=run):
                manifest = kernel.build(name, output, self.root / "cache", 2)
                first = kernel.validate_manifest(manifest, name)
                kernel.build(name, output, self.root / "cache", 2, force=True)
                second = kernel.validate_manifest(manifest, name)
            self.assertNotEqual(first["kernel_image"], second["kernel_image"])
            self.assertTrue(Path(first["kernel_image"]).is_file())
            self.assertEqual(commands[0][-1], kernel.TARGETS[name]["defconfig"])
            self.assertEqual(commands[1][-2:], ["-j2", kernel.TARGETS[name]["image"]])
            self.assertFalse(list(output.glob(".build-*")))


if __name__ == "__main__":
    unittest.main()
