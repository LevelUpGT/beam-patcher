<!--
Dosya Yolu: docs/INSTALLER-STRATEGY-TURKUAZ.md
Amac: Beam QA paketlemesinden TurkuazInstaller'a guvenli gecis plani ve kabul sinirlari
Modul - Markdown documentation
Version: 0.1.0-candidate
Aciklama: Inno Setup gecici QA artifact olarak kalir; imzali Turkuaz manifest ve Velopack E2E tamamlaninca gecis yapilir
Bagimli Oldugu Katman: installer/windows/BeamPatcher-QA.iss | TurkuazLabs/TurkuazInstaller/docs/PROJECT_INTEGRATION.md
-->

# Beam installer: Inno QA today, TurkuazInstaller target

## Why Inno was used

The October 25 Beam QA pipeline had a directly testable x64 Inno
Setup compilation/quiet-install/upgrade/uninstall chain. This was a
**scoped test artifact**, NOT an installer-platform decision. User-facing
production distribution should be tracked as a separate TurkuazInstaller
migration, not silently made dependent on the transient Inno QA flow.

Upstream community:
https://github.com/TurkuazLabs/TurkuazInstaller

Upstream project integration contract:
https://github.com/TurkuazLabs/TurkuazInstaller/blob/main/docs/PROJECT_INTEGRATION.md

The upstream Windows Desktop, Velopack E2E and local unsigned test
packaging checks are passing. But their unsigned local test ZIP cannot be
redistributed as a signed product installer.

## Required work before switching Beam installers

1. Pin permanent package ID `levelupgt.beampatcher` and per-user Windows x64
   destination. Decide whether QA remains a separate package identity
   `levelupgt.beampatcher.qa` before a public installer is issued.
2. Place writable `config.yml`, theme choice, game-directory selection and
   patch cache in user-writable app data, not versioned/replaceable Velopack
   `current`. Maintain explicit migration from existing Inno app directory.
3. Produce a **full Velopack nupkg + Setup.exe** using the actual Beam UI/EXE
   with the four local theme image assets, then verify correct WebView2
   rendering and theme selection after install.
4. Generate a versioned project manifest from the real artifact SHA-256,
   byte size, package/channel and per-user target. Do NOT commit placeholder
   artifact digest as a release manifest.
5. Sign the exact raw manifest with a detached CMS `.p7s` and separately
   provision package-scoped signer subject / certificate SHA256 pins in
   `%LOCALAPPDATA%/TurkuazInstaller/config/manifest-trust.yml`.
   Production TurkuazInstaller binaries and distributable artifacts need
   their corresponding signing and preflight approvals.
6. Execute isolated Windows clean install, same-version repair, next-version
   update, previous-version rollback, uninstall, shortcut and config
   preservation tests against a **signed test fixture**.
7. Verify non-admin install, Unicode/spaces in path, older Inno install
   migration, duplicate installed-app detection and tampered manifest/artifact
   refusal. Never target a real player's GRF for installer testing.
8. Only after all gates pass, retire Inno Setup from the production pipeline.
   The current QA script must remain available until replacement passes.

## Known current artifact state

- `installer/windows/BeamPatcher-QA.iss`: operational unsigned standalone
  QA installer. It is not a TurkuazInstaller package.
- `installer/windows/config.qa.yml`: offline/loopback-only, QA GRF;
  client launch and real game settings are disabled.
- `beam-ui/public/assets/1.png` through `4.png` are currently identical
  blobs. Separate approved LevelUp Games promotional art is an asset task,
  not a dependency on the remote Ragnarok patch feed.
- `b1glord/ragnarokclientpatch` is a game update channel containing THOR
  archives, not an approved source of launcher-specific banner PNG/JPG.
- Release signing keys/cert identities and real Ragnarok 2022 client
  compatibility are **not established** by the Beam QA tests.

The migration is **NOT IMPLEMENTED / NOT RELEASED**. Do not describe it as
production-ready until a separate pinned Turkuaz integration PR runs the
upstream signed manifest + full-package acceptance.
