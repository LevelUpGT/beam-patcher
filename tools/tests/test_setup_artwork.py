#!/usr/bin/env python3
# Dosya Yolu: tools/tests/test_setup_artwork.py
# Amac: Test artwork envanteri, SHA ve asama/ressam baglantisini dogrular
# Modul - Python unittest
# Version: 1.0.0
# Aciklama: Offline; gercek kurulumu calistirmaz ve gorseli internetten indirmez
# Bagimli Oldugu Katman: Build | Testing

import copy
import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1] / "windows/ragnar-turkuaz-qa"
sys.path.insert(0, str(ROOT / "branding"))
from validate_artwork import validate, resource_lines

CONFIG = json.loads((ROOT / "branding/backgrounds.json").read_text("utf-8"))
VIEW = (ROOT / "RagnarSetupWindow.cpp").read_text("utf-8")


class MultiArtworkTests(unittest.TestCase):
    def test_approved_source_hashes_and_page_mapping_are_pinned(self):
        self.assertEqual([x["resource_id"] for x in CONFIG["artworks"]], [102, 103, 104])
        self.assertEqual([x["asset_id"] for x in CONFIG["artworks"]], ["concept-c", "sunrise", "twilight"])
        self.assertEqual(CONFIG["artworks"][1]["steps"], ["options", "complete"])
        self.assertEqual(CONFIG["artworks"][2]["steps"], ["installing", "failed"])
        self.assertIn("case Page::Installing: index = 2;", VIEW)
        self.assertIn("case Page::Options: index = 1;", VIEW)
        self.assertIn("GetLastStatus()", VIEW)
        self.assertIn("--verify-artwork-present", VIEW)

    def test_no_image_fallback_is_identified_as_gradient(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = validate(Path(tmp), CONFIG)
            self.assertEqual(result["mode"], "gradient-only")
            self.assertEqual(result["embedded_count"], 0)
            self.assertEqual(resource_lines(Path(tmp), result), "")

    def test_partial_set_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "concept-c-background.png").write_bytes(b"x")
            with self.assertRaisesRegex(ValueError, "Partial artwork"):
                validate(Path(tmp), CONFIG)

    def test_corrupted_content_rejected_even_with_png_magic(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for record in CONFIG["artworks"]:
                fake = (b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\x0dIHDR" +
                        struct.pack(">II", record["width"], record["height"]) + b"\0"*1024)
                (root / record["file"]).write_bytes(fake)
            with self.assertRaisesRegex(ValueError, "SHA-256"):
                validate(root, CONFIG)

    def test_malicious_filename_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            changed = copy.deepcopy(CONFIG)
            changed["artworks"][0]["file"] = "../outside.png"
            with self.assertRaisesRegex(ValueError, "Unsafe"):
                validate(Path(tmp), changed)

    def test_nonapproved_resource_id_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            changed = copy.deepcopy(CONFIG)
            changed["artworks"][1]["resource_id"] = 105
            with self.assertRaisesRegex(ValueError, "Resource IDs"):
                validate(Path(tmp), changed)


if __name__ == "__main__":
    unittest.main()
