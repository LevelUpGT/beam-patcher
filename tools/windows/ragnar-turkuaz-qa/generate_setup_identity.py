#!/usr/bin/env python3
# Dosya Yolu: REPO_ROOT/tools/windows/ragnar-turkuaz-qa/generate_setup_identity.py
# Amac: Imzali metadata kaynagindan player-visible ad ve dahili profil kimligi uretir
# Modul - Python Build Tool
# Version: 1.0.0
# Aciklama: Real 2015/2022 client onaysiz QA preview olarak paketlenemez
# Bagimli Oldugu Katman: Build | Config | Native Windows Wizard

"""Generate a stable setup identity from repository-pinned client metadata."""

import argparse
import json
from pathlib import Path
import re
import sys

_SHA = re.compile(r"[a-f0-9]{40}\Z")
_SEMVER = re.compile(r"0|[1-9][0-9]*")
_SAFE_ID = re.compile(r"[a-z][a-z0-9-]{1,63}\Z")


def _version(version):
    parts = version.split(".")
    if len(parts) != 3 or not all(_SEMVER.fullmatch(x) for x in parts):
        raise ValueError("Installer version must be SemVer MAJOR.MINOR.PATCH")
    return version


def build_identity(data, profile_name, source_commit):
    if data.get("schema_version") != 1 or not _SHA.fullmatch(source_commit):
        raise ValueError("Invalid identity schema or source commit SHA")
    product = data["product"]
    if product["display_name"] != "Turkuaz Ragnarok Online":
        raise ValueError("Canonical display name must not be overridden")
    setup = product["public_setup_name"]
    if setup != "Turkuaz-Ragnarok-Online-Setup.exe":
        raise ValueError("Public filename must remain stable and omit client details")
    version = _version(product["installer_semver"])
    if not _SAFE_ID.fullmatch(profile_name):
        raise ValueError("Invalid client profile id")
    profile = data["profiles"][profile_name]
    if profile_name != "preview-qa" or not profile["qa_install_enabled"]:
        raise ValueError("Only the read-only preview is authorized by this QA build")
    if profile["production_publish_allowed"] or profile["client_year"] is not None or profile["packetver"] is not None or profile["client_build"] is not None:
        raise ValueError("A visual preview cannot claim an actual game client version")
    if profile["channel"] != "qa" or profile["client_family"] != "preview":
        raise ValueError("Preview client metadata mismatch")
    if profile["installed_main_exe"] != "RagnarClassicPreview.exe":
        raise ValueError("Unsafe or unexpected preview main EXE")
    if profile["install_directory"] != "RagnarClassicTurkuazQA":
        raise ValueError("Existing QA install path must stay compatible")
    if profile["velopack_pack_id"] != "levelupgt.ragnar-classic-preview":
        raise ValueError("Existing preview Velopack identity must stay compatible")
    # Fields stay distinct: game-client version, packet version and installer SemVer.
    return {
        "schema_version": 1,
        "display_name": product["display_name"],
        "public_setup_name": setup,
        "profile_id": profile_name,
        "client_family": profile["client_family"],
        "client_year": None,
        "client_build": None,
        "packetver": None,
        "installer_version": version,
        "channel": profile["channel"],
        "source_commit": source_commit,
        "artifact_id": f"turkuaz-ragnarok-online-{profile_name}-{version}-{source_commit[:12]}-test-only",
        "velopack_pack_id": profile["velopack_pack_id"],
        "main_exe": profile["installed_main_exe"],
        "install_directory": profile["install_directory"],
        "production_publish_allowed": False,
    }


def _wstring(text):
    if not isinstance(text, str) or any(ord(c) < 32 or ord(c) > 126 for c in text):
        raise ValueError("Generated C++ values must use safe printable ASCII")
    return 'L"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def write_outputs(identity, header, report):
    if header.resolve() == report.resolve():
        raise ValueError("Header and report paths cannot coincide")
    for dest in (header, report):
        if not dest.parent.is_dir():
            raise ValueError(f"Missing output parent directory: {dest.parent}")
    macros = {
        "TRO_PRODUCT_NAME": identity["display_name"],
        "TRO_WINDOW_TITLE": identity["display_name"] + " - Kurulum",
        "TRO_INSTALLER_VERSION": identity["installer_version"],
        "TRO_PROFILE_ID": identity["profile_id"],
        "TRO_SETUP_EXE_NAME": identity["public_setup_name"],
        "TRO_MAIN_EXE": identity["main_exe"],
    }
    content = ("// Generated from client-profiles.json; do not edit or include private keys.\n"
               "#pragma once\n" + "".join(
                   f"#define {key} {_wstring(value)}\n" for key, value in macros.items()
               ))
    # Both output files are QA worktree-generated artifacts, never publish credentials.
    header.write_text(content, encoding="utf-8")
    report.write_text(json.dumps(identity, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default="preview-qa")
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--config", type=Path, default=Path(__file__).with_name("client-profiles.json"))
    parser.add_argument("--header", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        data = json.loads(args.config.read_text("utf-8"))
        result = build_identity(data, args.profile, args.source_commit.lower())
        write_outputs(result, args.header, args.report)
    except (KeyError, TypeError, OSError, ValueError, json.JSONDecodeError) as error:
        print(f"SETUP_IDENTITY_REJECTED: {error}", file=sys.stderr)
        return 2
    print(f"SETUP_IDENTITY_OK id={result['artifact_id']} profile={result['profile_id']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
