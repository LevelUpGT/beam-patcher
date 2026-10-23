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

## Windows 10 ordinal 345 startup regression (2026-10-24)

Earlier one-file QA EXE could fail on Windows 10 at process load time
with "ordinal 345 could not be located". Ordinal 345 corresponds to
TaskDialogIndirect exported from ComCtl32 v6. The old build accidentally
relied on static imports without independently verifying the selected
Common Controls assembly in the end-user installer.

The corrected portable single EXE:
- embeds `ragnar-setup.manifest` as resource type 24, ID 1,
  explicitly requesting Microsoft.Windows.Common-Controls v6;
- resolves `TaskDialogIndirect` dynamically by name and never imports
  ComCtl32 ordinal 345 at EXE process initialization;
- rejects unavailable Task Dialog APIs with a readable error instead of
  a Windows loader failure;
- exercises two separate Windows CI checks: manifest extraction from the
  **final** EXE, then live `--verify-dialog-runtime` import resolution;
- continues to check the embedded Velopack payload SHA-256;
- does not claim a Windows 10 GUI smoke unless actually performed on
  a real Windows 10 test environment.

The previously published EXE with SHA-256
`aa2c2903376da4c1e524340285a391285994715e8e8cfb68c17b0d81ed2fa9f8`
is superseded and should not be used for testing.

## Single-window installer UX (QA v0.3.0)

The full-screen-style splash is **the installer**, not an additional
Velopack splash. It is one persistent Windows native window with:
Welcome -> Options -> Installing -> Complete/Failed. Only native
Windows folder picking and actionable error dialogs can open a separate
system dialog. It does not copy Adobe artwork.

The native front end invokes the embedded, SHA-256 checked Velopack
Setup.exe using **`--silent --installto`**. Velopack's own splash and
post-install automatic app start are suppressed. The preview main
executable also short-circuits `--veloapp-*` lifecycle hooks so that
Velopack install hooks never open Beam's normal window. The preview
program can be launched **only after completion** when the user clicks
"RAGNAR'I AÇ".

`vpk pack --shortcuts None` disables Velopack-managed automatic
shortcuts in this test package. The user can opt into Desktop, Start
Menu, both, or neither. Shortcut creation is performed only after
successful installation with the Windows ShellLink APIs. Existing
shortcuts owned by other programs/users are not overwritten. Tests
must inspect real Windows shortcut behavior and future update/repair
semantics before any production rollout.

The progress animation is **indeterminate** by design: Velopack 1.2
Setup.exe exposes no authoritative fractional progress through this
wrapper. No fabricated completion percentage is displayed. During
installation the window cannot be dismissed with the close button;
the worker waits for the process exit and reports its real status.
On failure, Velopack's local log location is shown in the error page.
CI checks the view's real hidden-window HWND/controls with
`--verify-ui-contract` but does not claim this is an end-user
Windows 10 pixel smoke.
