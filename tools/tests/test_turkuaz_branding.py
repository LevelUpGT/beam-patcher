#!/usr/bin/env python3
# Dosya Yolu: REPO_ROOT/tools/tests/test_turkuaz_branding.py
# Amac: Turkuaz Ragnarok Online gorunur marka ve QA uyumluluk kontrati
# Modul - Python unittest
# Version: 1.0.0-candidate
# Aciklama: Eski tema ID'lerini ve izole QA profilini koruyarak gorunur adi denetler
# Bagimli Oldugu Katman: beam-ui/public/index.html | beam-core/src/config.rs | installer/windows

import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
HTML = (ROOT / "beam-ui/public/index.html").read_text(encoding="utf-8")
TAURI = json.loads((ROOT / "beam-ui/tauri.conf.json").read_text(encoding="utf-8"))
QA = (ROOT / "installer/windows/config.qa.yml").read_text(encoding="utf-8")
CONFIG = (ROOT / "beam-core/src/config.rs").read_text(encoding="utf-8")
GUARD = (ROOT / "beam-core/src/qa_guard.rs").read_text(encoding="utf-8")
NSIS = (ROOT / "installer/windows/BeamPatcher-NSIS-Backup.nsi").read_text(encoding="utf-8")
DUAL = (ROOT / ".github/workflows/dual-installer-policy.yml").read_text(encoding="utf-8")
WIN = (ROOT / ".github/workflows/windows-setup-patch-qa.yml").read_text(encoding="utf-8")
VERIFY = (ROOT / "tools/verify_installer_pair.py").read_text(encoding="utf-8")


class TurkuazBrandingTests(unittest.TestCase):
    def test_launcher_identity_is_not_goatmmo_or_levelup(self):
        self.assertIn("<title>Turkuaz Ragnarok Online</title>", HTML)
        self.assertIn('id="logo-text">TURKUAZ RAGNAROK ONLINE</span>', HTML)
        self.assertIn('aria-label="Turkuaz Ragnarok Online"', HTML)
        self.assertIn("TURKUAZ · RAGNAROK ONLINE", HTML)
        for old in ("GOATMMO", "LEVELUP GAMES", "goatmmo.com",
                    "Ragnar Ragnarok Online", "LevelUp Games"):
            self.assertNotIn(old, HTML)
        self.assertEqual(TAURI["package"]["productName"], "Turkuaz Ragnarok Online")
        self.assertIn("Turkuaz Ragnarok Online", TAURI["tauri"]["windows"][0]["title"])
        # Keep platform identity stable until a separate signed migration.
        self.assertEqual(TAURI["tauri"]["bundle"]["identifier"], "com.beampatcher.app")

    def test_four_display_names_keep_original_persistent_theme_keys(self):
        names = {
            "default": "Turkuaz Standart",
            "ragnar-legacy": "Turkuaz Nostalji",
            "ragnar-classic": "Turkuaz Klasik",
            "levelup-modern": "Turkuaz Modern",
        }
        for key, name in names.items():
            self.assertIn(f'data-launcher-theme="{key}"', HTML)
            self.assertIn(f'<option value="{key}">{name}</option>', HTML)
            self.assertIn(f'class="theme-choice-name">{name}</span>', HTML)
        self.assertEqual(HTML.count('data-launcher-theme="'), 4)
        self.assertIn("'beam.launcher.theme.v1'", HTML)
        self.assertIn("'about:blank'", HTML)

    def test_existing_and_new_qa_names_remain_isolated(self):
        self.assertIn('name: "Turkuaz Ragnarok Online QA"', QA)
        self.assertIn('server_name: "TURKUAZ RAGNAROK ONLINE (QA)"', QA)
        self.assertIn("qa_only: true", QA)
        self.assertIn('client_exe: "NO_CLIENT_CONFIGURED.exe"', QA)
        self.assertIn('target_grf: "levelup-qa.grf"', QA)
        self.assertIn("enabled: false", QA)
        self.assertIn('QA_APP_NAME: &str = "Turkuaz Ragnarok Online QA"', GUARD)
        self.assertIn('LEGACY_QA_APP_NAME: &str = "LevelUp Games Beam QA"', GUARD)
        self.assertIn("is_qa_app_name(&config.app.name)", CONFIG)
        self.assertIn("is_qa_app_name(&self.app.name)", CONFIG)
        self.assertIn("config.patcher.qa_only = true;", CONFIG)

    def test_nsis_stays_private_and_matches_windows_ci(self):
        setup = "TurkuazRagnarokOnline-NSIS-BACKUP-QA-Setup.exe"
        folder = "Programs/TurkuazSoft/Turkuaz Ragnarok Online QA-NSIS-Backup"
        self.assertIn(setup, NSIS)
        self.assertIn(setup, DUAL)
        self.assertIn(setup, WIN)
        self.assertIn(setup, VERIFY)
        self.assertIn(folder, DUAL)
        self.assertIn(folder, WIN)
        self.assertIn("NSIS BACKUP ONLY", NSIS)
        self.assertIn("Turkuaz Ragnarok Online QA", NSIS)
        self.assertIn("verify_installer_pair.py", DUAL)
        self.assertNotIn("LevelUpGames-Beam-NSIS-BACKUP-QA-Setup.exe", DUAL)


if __name__ == "__main__":
    unittest.main()
