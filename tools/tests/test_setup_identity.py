#!/usr/bin/env python3
# Dosya Yolu: REPO_ROOT/tools/tests/test_setup_identity.py
# Amac: Installer SemVer, istemci profili ve QA dagitim sinirini dogrular
# Modul - Python unittest
# Version: 1.0.0
# Aciklama: Offline only, 2015/2022 gercek oyun istemcisi olarak sunulamaz
# Bagimli Oldugu Katman: Config | QA Build

import copy
import json
from pathlib import Path
import tempfile
import unittest
import sys

ROOT = Path(__file__).resolve().parents[1] / "windows/ragnar-turkuaz-qa"
sys.path.insert(0, str(ROOT))
from generate_setup_identity import build_identity, write_outputs

CONFIG = json.loads((ROOT / "client-profiles.json").read_text("utf-8"))
COMMIT = "a" * 40


class ClientIdentityTests(unittest.TestCase):
    def test_preview_identity_is_not_an_actual_game_client(self):
        record = build_identity(CONFIG, "preview-qa", COMMIT)
        self.assertEqual(record["display_name"], "Turkuaz Ragnarok Online")
        self.assertEqual(record["public_setup_name"], "Turkuaz-Ragnarok-Online-Setup.exe")
        self.assertEqual(record["profile_id"], "preview-qa")
        self.assertIsNone(record["client_year"])
        self.assertIsNone(record["packetver"])
        self.assertFalse(record["production_publish_allowed"])
        self.assertEqual(record["velopack_pack_id"], "levelupgt.ragnar-classic-preview")
        self.assertIn(COMMIT[:12], record["artifact_id"])

    def test_2015_and_2022_cannot_be_selected_before_client_verification(self):
        for key in ("classic-2015", "renewal-2022"):
            with self.subTest(profile=key), self.assertRaises(ValueError):
                build_identity(CONFIG, key, COMMIT)
        self.assertEqual(CONFIG["profiles"]["classic-2015"]["client_year"], 2015)
        self.assertEqual(CONFIG["profiles"]["renewal-2022"]["client_year"], 2022)
        self.assertFalse(CONFIG["profiles"]["renewal-2022"]["production_publish_allowed"])

    def test_forged_preview_client_year_rejected(self):
        data = copy.deepcopy(CONFIG)
        data["profiles"]["preview-qa"]["client_year"] = 2022
        with self.assertRaises(ValueError):
            build_identity(data, "preview-qa", COMMIT)

    def test_invalid_or_wrong_commit_rejected(self):
        with self.assertRaises(ValueError):
            build_identity(CONFIG, "preview-qa", "a" * 7)

    def test_unsigned_publish_enablement_rejected(self):
        data = copy.deepcopy(CONFIG)
        data["profiles"]["preview-qa"]["production_publish_allowed"] = True
        with self.assertRaises(ValueError):
            build_identity(data, "preview-qa", COMMIT)

    def test_cpp_header_and_json_are_derived_from_same_source(self):
        record = build_identity(CONFIG, "preview-qa", COMMIT)
        with tempfile.TemporaryDirectory() as folder:
            header, report = Path(folder) / "identity.h", Path(folder) / "identity.json"
            write_outputs(record, header, report)
            self.assertIn('#define TRO_PRODUCT_NAME L"Turkuaz Ragnarok Online"', header.read_text("utf-8"))
            self.assertIn('#define TRO_INSTALLER_VERSION L"0.1.0"', header.read_text("utf-8"))
            self.assertEqual(json.loads(report.read_text("utf-8")), record)


if __name__ == "__main__":
    unittest.main()
