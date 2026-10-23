<!--
Dosya Yolu: tools/windows/ragnar-turkuaz-qa/README.md
Amac: Ragnar Classic preview kurulumunu TurkuazInstaller Community ile test etme kilavuzu
Modul - Markdown
Version: 0.1.0-qa
Aciklama: Uretim release olmadan WinUI marka profili, test CMS, Velopack install ve cleanup adimlari
Bagimli Oldugu Katman: Tool | TurkuazInstaller Community | Beam preview-only
-->

# Ragnar Classic - Turkuaz Installer QA

This isolated integration uses **TurkuazLabs/TurkuazInstaller Community**,
not Inno Setup, to install a **read-only** Ragnar Classic Beam visual
preview. It does not install or patch the actual Ragnarok game.

## Components

- Beam Tauri/Rust preview-only app, with its patch/game APIs unregistered.
- Velopack 1.2.161 Setup.exe as the full install artifact accepted by
  TurkuazInstaller's package engine.
- Pinned Community core/NativeAOT Bootstrapper/WinUI/CLI at commit
  1048931bf7498ab4d4440414c4463e6ee5e7421e.
- Ragnar-branded ui-profile.yml using supported tr-TR text extension
  (window title, header, footer, localization). This is **not** a new
  XAML theme or a custom skin of the TurkuazInstaller engine.
- A self-contained QA provisioner that computes the *actual local*
  Setup.exe SHA-256/size, signs a generated manifest with an **ephemeral
  local QA certificate**, and provisions its exact certificate pin in
  the external per-user TurkuazInstaller trust policy.

Production signature and independent publisher identity are **NOT**
provided by the QA signer. The QA signer private key is never saved or
uploaded. Do not use it as a real publisher trust root.

## Artifact layout

- Start-RagnarTurkuazQA.cmd / Uninstall-RagnarTurkuazQA.cmd
- TurkuazInstaller.Bootstrapper.exe
- app/TurkuazInstaller.WinUI.exe (+ dependencies)
- cli/TurkuazInstaller.Cli.exe (+ dependencies)
- qa/Ragnar.QaProvisioner.exe
- qa/ui-profile.yml
- payload/RagnarClassicPreview-Setup.exe
- SHA256SUMS.txt

No Inno .iss or original 2015 THOR binary is used in this build.

## Safety and usage

**Only run this unsigned QA build in a disposable Windows x64 VM or a
clean, dedicated Windows QA user. Do not bypass SmartScreen or antivirus.**

1. Extract the entire short-lived GitHub Actions artifact ZIP to a
   normal writable folder. Do not execute an EXE within a ZIP.
2. Verify its checksum against workflow metadata / SHA256SUMS.
3. Double-click Start-RagnarTurkuazQA.cmd.
4. Provisioner refuses to overwrite an existing global TurkuazInstaller
   UI profile or manifest trust policy. Never delete an existing user's
   configuration to force this QA through.
5. With a fresh QA profile it creates an exact local-file manifest,
   .p7s detached CMS signature, and external certificate pin. Then
   TurkuazInstaller CLI invokes the real Velopack install path,
   checks metadata and payload SHA-256, and opens the branded GUI.
6. Preview app installs into %LOCALAPPDATA%/Programs/RagnarClassicTurkuazQA
   (never into the game directory).
7. Uninstall-RagnarTurkuazQA.cmd uses TurkuazInstaller uninstall and then
   removes only the two QA-owned profile/trust files.

The local generated manifest lives next to the QA provisioner, not in Git.
Re-running Start does not silently overwrite/re-sign it. If the first
install fails after preparation, inspect diagnostics and use the documented
TurkuazInstaller CLI with the existing signed manifest in the isolated QA
account; do not repeatedly re-provision.

## Gates and limitations

Before artifact upload the Windows workflow checks:
- Classic/Legacy theme source tests (not a GUI pixel test).
- Native Rust preview-only Windows x64 build.
- Real Velopack package generation.
- Pinned TurkuazInstaller Community Windows distribution.
- QA provisioner's real CMS detached signature, external pin,
  TurkuazInstaller CLI install and uninstall.
- Package inventory and SHA256.

UI pixel/launcher screenshot QA on the user's machine and production
publisher Authenticode are NOT RUN by the integration tests.
Brandable fields cover text, not image backgrounds; a separate TurkuazInstaller
theming feature would require a reviewed upstream framework change.

**No release, no player update feed, no GRF, no DATA.INI, no game
executable changes.** Existing Inno test workflow remains untouched.
