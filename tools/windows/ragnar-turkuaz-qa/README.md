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

## Artifact layout: one installer for the user

The previous 177 MB developer ZIP carried .NET/WinUI, TurkuazInstaller CLI,
provisioner, and a nested Velopack Setup.exe. That ZIP is a **QA development
bundle**, not an appropriate user installer.

The replacement Actions artifact contains only:

- RagnarClassic-Turkuaz-Setup.exe (standalone x64 native Windows wizard)
- CHECKSUM.txt (SHA-256 metadata)

Actions automatically delivers these files inside an outer ZIP. There is no
nested distribution ZIP, no app/ or cli/ folder and no developer executable
to select. The standalone EXE embeds the same Velopack Setup.exe that the
TurkuazInstaller Community CLI verified during the hosted QA pipeline.
The exact embedded payload's SHA-256 is checked again by the wizard before
any install.

The lightweight setup **does not embed the full TurkuazInstaller Community
Core, CLI or WinUI**. Its wizard is a separate native Windows front-end, with
Velopack Setup.exe as its per-user package engine. The TurkuazInstaller
Community signed-manifest/CLI integration is exercised in CI, but the
small single-EXE end-user path itself is **not yet TurkuazInstaller Core**.
A fully self-contained one-EXE product using TurkuazInstaller Core itself
would require a dedicated native product bootstrap/embedding feature in
the upstream installer framework and a significantly larger payload.

## User-facing install wizard

1. Welcome/preview-only warning in Turkish.
2. Default install path in per-user LocalAppData, with optional folder picker.
   Custom paths are isolated under a RagnarClassicTurkuazQA subfolder;
   Windows system directory and invalid paths are rejected.
3. Choose shortcuts: Desktop+Start, Start only, Desktop only, or None.
   Velopack creates its standard desktop/Start shortcuts; the wizard
   removes declined shortcut destinations only after verifying that
   the .lnk points into the new preview installation. The setup must not
   touch other applications' shortcuts.
4. Confirm, then launch the original Velopack Setup.exe. The packager
   now includes an original Ragnar splash image and icon.
5. Report success/failure and clean temporary embedded Setup resources.

The native EXE is unsigned QA software. It must only be opened in a
disposable Windows test VM; do not disable antivirus or SmartScreen.
There is no GRF/data.ini/real game installation or production release.
A fresh real-Windows GUI screenshot smoke, custom folder install,
all 4 shortcut modes, uninstall cleanup, and upgrade behavior are **not
automatically proven** by a source/build check.

Icon replacement is supported by adding an authorized ICO file to:
tools/windows/ragnar-turkuaz-qa/branding/icon.ico
before a future rebuild. Without an override the existing Beam icon is
used. The splash uses build-time original text/artwork.

## Internal TurkuazInstaller QA (not distributed)

The Windows pipeline still builds the pinned TurkuazInstaller Community
Bootstrapper, WinUI and CLI in a temporary CI directory. It exercises
the real signed local manifest + detached CMS + external trust configuration,
artifact SHA-256 and Velopack install/uninstall. Its private QA signer
is ephemeral and discarded, and private signing secrets are not published.
The binary of that internal test environment is excluded from the user artifact.

## Gates and limitations

Before artifact upload, the Windows workflow checks:
- Classic/Legacy theme source tests (not a GUI pixel test).
- Native Rust preview-only Windows x64 build.
- Real Velopack package generation.
- Pinned TurkuazInstaller Community Windows distribution.
- QA provisioner's real CMS detached signature, external pin,
  TurkuazInstaller CLI install and uninstall.
- Package inventory and SHA256.

Native wizard UI screenshot QA on the user's machine and production
publisher Authenticode are NOT RUN by the integration tests.
Product-specific splash is embedded into Velopack; true custom TurkuazInstaller
WinUI theming still requires an upstream framework feature.

**No release, no player update feed, no GRF, no DATA.INI, no game
executable changes.** Existing Inno test workflow remains untouched.
