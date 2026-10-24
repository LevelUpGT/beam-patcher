#!/usr/bin/env python3
# Dosya Yolu: tools/windows/ragnar-turkuaz-qa/branding/validate_artwork.py
# Amac: Onayli Setup arka planlarinin SHA-256, PNG geometri ve kaynak envanterini kilitler
# Modul - Python Build Tool
# Version: 1.0.0
# Aciklama: Tam uc gorsel ya da fallback; kismi/gecersiz dosyada hata verir
# Bagimli Oldugu Katman: Build | Security | Packaging | View

"""Offline validation of artwork before embedding in a Win32 setup resource."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

_SHA = re.compile(r"[a-f0-9]{64}\Z")
_SAFE_NAME = re.compile(r"[a-z][a-z0-9-]{1,48}\.png\Z")
_PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
_MAX_PNG = 16 * 1024 * 1024
_RESOURCE_IDS = (102, 103, 104)
_ASSET_IDS = ("concept-c", "sunrise", "twilight")


def validate(root: Path, manifest: dict):
    records = manifest.get("artworks")
    if manifest.get("schema_version") != 1 or not isinstance(records, list) or len(records) != 3:
        raise ValueError("Invalid reviewed artwork manifest schema")
    if [r.get("resource_id") for r in records] != list(_RESOURCE_IDS):
        raise ValueError("Resource IDs must match native wizard contract")
    if [r.get("asset_id") for r in records] != list(_ASSET_IDS):
        raise ValueError("Artworks must use reviewed order")
    present = []
    for item in records:
        name = item["file"]
        if not isinstance(name, str) or not _SAFE_NAME.fullmatch(name):
            raise ValueError("Unsafe or unexpected artwork file name")
        if not isinstance(item.get("sha256"), str) or not _SHA.fullmatch(item["sha256"]):
            raise ValueError("Invalid expected artwork digest")
        path = root / name
        if path.is_symlink():
            raise ValueError("Symlink artwork is not allowed")
        present.append(path.is_file())
    if any(present) and not all(present):
        raise ValueError("Partial artwork set: fail closed rather than ship wrong images")
    embedded = all(present)
    result = []
    for item in records:
        path = root / item["file"]
        actual = None
        size = 0
        if embedded:
            size = path.stat().st_size
            if not (1024 <= size <= _MAX_PNG):
                raise ValueError("Artwork PNG size exceeds policy")
            digest = hashlib.sha256()
            with path.open("rb") as source:
                header = source.read(24)
                if len(header) < 24 or header[:8] != _PNG_SIGNATURE or header[12:16] != b"IHDR":
                    raise ValueError("Invalid PNG signature or IHDR")
                width, height = struct.unpack(">II", header[16:24])
                if (width, height) != (item["width"], item["height"]):
                    raise ValueError("Artwork PNG dimensions do not match approved source")
                digest.update(header)
                for block in iter(lambda: source.read(1024 * 1024), b""):
                    digest.update(block)
            actual = digest.hexdigest()
            if actual != item["sha256"]:
                raise ValueError("Artwork content differs from approved SHA-256")
            if path.stat().st_size != size:
                raise ValueError("Artwork changed during SHA-256 inspection")
        result.append({
            "asset_id": item["asset_id"], "file": item["file"], "resource_id": item["resource_id"],
            "expected_sha256": item["sha256"], "sha256": actual,
            "size_bytes": size, "embedded": embedded, "steps": item["steps"]
        })
    return {
        "schema_version": 1,
        "mode": "approved-artwork" if embedded else "gradient-only",
        "embedded_count": len(result) if embedded else 0,
        "artworks": result,
        "qa_only": True,
        "production_acceptance": "NOT_ACCEPTED",
        "warning": "Artwork identity verification is not malware clearance, GUI pixel acceptance or release signing."
    }


def resource_lines(root: Path, report: dict) -> str:
    if report["mode"] == "gradient-only":
        return ""
    lines = []
    for item in report["artworks"]:
        name = str((root / item["file"]).resolve())
        if '"' in name:
            raise ValueError("Unsafe artwork path")
        # Windows rc.exe resource strings need doubled separators.
        lines.append(f'{item["resource_id"]} RCDATA "{name.replace(chr(92), chr(92)*2)}"')
    return "\n".join(lines) + "\n"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--rc-output", type=Path, required=True)
    parser.add_argument("--json-output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        root = args.root.resolve(strict=True)
        manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
        evidence = validate(root, manifest)
        content = resource_lines(root, evidence)
        for path in (args.rc_output, args.json_output):
            if not path.parent.is_dir():
                raise ValueError("Artwork output directory must already exist")
            if path.exists():
                raise ValueError("Refusing to overwrite existing artwork QA evidence")
        args.rc_output.write_text(content, encoding="utf-8")
        args.json_output.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
        print(f"TRO_ARTWORK_CONTRACT_OK mode={evidence['mode']} count={evidence['embedded_count']}")
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as exc:
        print(f"TRO_ARTWORK_CONTRACT_FAILED: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
