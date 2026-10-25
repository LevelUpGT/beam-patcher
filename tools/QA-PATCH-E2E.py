#!/usr/bin/env python3
# Dosya Yolu: tools/QA-PATCH-E2E.py
# Amac: Yerel HTTP BEAM patchi CLI ile gercek GRF dosyasina uygular, SHA/cache korumasini sinar
# Modul - Python integration QA harness
# Version: 0.1.0-candidate
# Aciklama: Her calismada yeni gecici klasor, sentetik harita, localhost, no live client
# Bagimli Oldugu Katman: tools/tests/prepare_rust_beam_fixture.py | beam-patcher/src/main.rs

"""Run real BEAM download/apply/readback smoke ONLY against a disposable synthetic GRF."""

import argparse
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
from threading import Thread

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "installer/windows/config.qa.yml"
FIXTURE_BUILDER = ROOT / "tools/tests/prepare_rust_beam_fixture.py"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def invoke(executable, config_path, should_pass):
    completed = subprocess.run(
        [str(executable), "--headless", "--config", str(config_path)],
        cwd=config_path.parent,
        text=True, capture_output=True, timeout=120, check=False,
    )
    if (completed.returncode == 0) != should_pass:
        raise AssertionError(
            f"headless patch expected success={should_pass} got {completed.returncode}; "
            f"stderr tail: {completed.stderr[-1500:]}"
        )
    return completed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, required=True, help="Built Beam executable")
    args = parser.parse_args()
    exe = args.exe.resolve()
    if not exe.is_file():
        parser.error(f"EXE is missing: {exe}")
    if not PROFILE.is_file() or not FIXTURE_BUILDER.is_file():
        parser.error("QA profile/fixture builder missing")

    with tempfile.TemporaryDirectory(prefix="levelup-beam-patch-qa-") as root:
        base = Path(root)
        fixture = base / "fixture"
        publish = base / "web"
        game = base / "isolated-client"
        publish.mkdir()
        game.mkdir()

        # Generated data are synthetic, never copied from a real client.
        subprocess.run([sys.executable, str(FIXTURE_BUILDER), str(fixture)],
                       cwd=str(ROOT), check=True, capture_output=True, text=True, timeout=60)
        archive = publish / "qa-maps.beam"
        shutil.copyfile(fixture / "custom-maps.beam", archive)
        (publish / "patchlist.txt").write_text(
            f"qa-maps.beam {sha256(archive)} {archive.stat().st_size}\n", encoding="utf-8"
        )

        server = ThreadingHTTPServer(
            ("127.0.0.1", 0),
            functools.partial(SimpleHTTPRequestHandler, directory=str(publish)),
        )
        worker = Thread(target=server.serve_forever, daemon=True)
        worker.start()
        try:
            uri = f"http://127.0.0.1:{server.server_address[1]}"
            qa_text = PROFILE.read_text(encoding="utf-8")
            assert "game_directory: null" in qa_text
            assert qa_text.count("http://127.0.0.1:9/disabled") >= 2
            qa_text = qa_text.replace("game_directory: null",
                                      f"game_directory: {json.dumps(str(game))}")
            qa_text = qa_text.replace("http://127.0.0.1:9/disabled", uri)
            config_path = base / "qa-config.yml"
            config_path.write_text(qa_text, encoding="utf-8")

            first = invoke(exe, config_path, should_pass=True)
            grf = game / "levelup-qa.grf"
            if not grf.is_file():
                raise AssertionError("GRF was not created by actual patcher")
            content = grf.read_bytes()
            if content[:15] != b"Master of Magic":
                raise AssertionError("Invalid GRF header")
            version = struct.unpack_from("<I", content, 42)[0]
            if version != 0x200:
                raise AssertionError(f"Unexpected GRF version {version:#x}")
            entry_count = struct.unpack_from("<I", content, 38)[0]
            if entry_count != 6:
                raise AssertionError(f"Unexpected GRF entry count {entry_count}, wanted 6")
            cache = game / ".patch_cache/applied_patches.txt"
            if not cache.is_file() or cache.read_text().splitlines() != ["qa-maps.beam"]:
                raise AssertionError("Applied cache not recorded after successful GRF save")
            before = sha256(grf)
            invoke(exe, config_path, should_pass=True)
            if sha256(grf) != before:
                raise AssertionError("Second run unexpectedly rewrote unchanged GRF")

            # SHA mismatch must be rejected before any GRF write or cache mark.
            (publish / "bad.beam").write_bytes(b"untrusted corrupted fixture")
            (publish / "patchlist.txt").write_text(
                f"qa-maps.beam {sha256(archive)}\n"
                f"bad.beam {'a' * 64}\n", encoding="utf-8",
            )
            invoke(exe, config_path, should_pass=False)
            if sha256(grf) != before:
                raise AssertionError("Failed checksum changed the verified GRF")
            if cache.read_text().splitlines() != ["qa-maps.beam"]:
                raise AssertionError("Failed checksum marked patch as applied")
            if (game / "levelup-qa.grf.bak").exists():
                raise AssertionError("GRF left a backup after aborted download")
            print(json.dumps({
                "schema": "levelupgt.beam.patch-e2e-qa.v1",
                "source": "synthetic generated BEAM archive, local HTTP only",
                "status": "PASS",
                "grf_version": "0x200",
                "grf_entries": entry_count,
                "first_patch_grf_sha256": before,
                "rerun_idempotent": True,
                "corrupt_patch_rejected": True,
                "cache_protected": True,
                "live_game_test": "NOT RUN",
                "published": False,
            }, sort_keys=True))
        finally:
            server.shutdown()
            server.server_close()
            worker.join(timeout=5)


if __name__ == "__main__":
    main()
