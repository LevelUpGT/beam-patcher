<!--
Dosya Yolu: docs/WINDOWS-SETUP-PATCH-QA.md
Amac: Beam QA kurulumu, manuel BEAM patch ve Windows artifacts icin kabul sinirlari
Modul - Markdown deployment/testing guide
Version: 0.2.0-candidate
Aciklama: 2022 public feed acilmadan kullanilabilen installer ve sentetik patch E2E
Bagimli Oldugu Katman: .github/workflows/windows-setup-patch-qa.yml | installer/windows/BeamPatcher-QA.iss
-->

# Turkuaz Ragnarok Online - Windows Setup & Patch QA

**Status: Candidate / NOT production.** The Windows x64 installer is deliberately
separate from a real Ragnarok client installer. It contains the actual
`beam-patcher.exe` Tauri/Rust launcher, four theme styles, a safe YAML config
and an uninstall entry. **No licensed client EXE, GRF, DATA.INI, patch feed,
SSL keys, credentials or game binaries are bundled.**

## QA installation

1. Open the [Windows Setup and Patch QA Actions workflow](../.github/workflows/windows-setup-patch-qa.yml)
   for the specific head commit on PR #13.
2. Download the short-lived `levelup-beam-windows-setup-qa-<SHA>` artifact
   **only when BOTH Windows test job and installer smoke are PASS**.
3. Verify `setup-qa-report.json` SHA256 against the extracted
   `LevelUpGames-BeamPatcher-QA-Setup.exe`. It is an unsigned test build.
4. Install as the current Windows user. No admin access is required.
5. Create a separate, disposable QA directory. Never select the running
   `D:\Ragnarok` or the installation containing personal game data.
6. Launch Beam QA. When prompted, choose the disposable game directory.
   Theme gallery lives in **Appearance / Settings**. Live news is not required.
7. QA default output is `levelup-qa.grf`, a new unregistered 0x200 GRF.
   The **Manual Patch** button accepts `.beam`, `.thor`, `.rgz`, `.gpf`.
   Loose-file THORs are rejected; their transaction-safe client writer is
   not implemented. Normal patch feed is disabled: `127.0.0.1:9/disabled`.
   The launcher **never enables Play** in QA mode. Appearance Settings exposes
   **Test Klasoru Sec** for selecting a fresh, empty workspace; a successful
   manual patch reports success in the patch progress label.

   **Code-enforced QA isolation** (`patcher.qa_only: true`) accepts only an
   absolute existing directory that is empty or contains only
   `levelup-qa.grf` and the expected `.patch_cache/applied_patches.txt`.
   Existing DATA.INI, game EXE/GRF, arbitrary files and symlink/junction
   paths are rejected before any new cache or GRF is created.
   All remote patchlist/mirror URLs must be HTTP loopback (`127.0.0.1`);
   production feeds, other GRF names, missing checksums and a configured
   game EXE are rejected. This is an accidental-damage protection policy,
   not a tamper-proof sandbox against someone editing the configuration.
8. Uninstall from Windows settings; external QA GRFs and edited user
   config remain intentionally intact. Inspect files before deletion.

**Note:** The launcher `client_exe` is deliberately
`NO_CLIENT_CONFIGURED.exe`, so the QA package does not launch a real game.
To bind a real client requires verified executable identity and region
compatibility; do not simply set the active game directory in this package.

## Reproducible test patch

The CI runs `tools/QA-PATCH-E2E.py` against **actual** compiled
`beam-patcher.exe` with a generated synthetic `.beam` and localhost HTTP
server. Expected assertions:

- Download BEAM with complete SHA-256 and apply into disposable GRF 0x200.
- Read GRF magic, version and number of synthetic entries.
- Re-run without changing already applied content, using the cache.
- Apply the same BEAM manually into another disposable GRF.
- Offer a corrupted BEAM with declared bad digest: reject it; target GRF hash
  and applied-patch cache remain unchanged.
- Reject a directory containing a simulated production DATA.INI and a remote
  update feed before any GRF/cache I/O.
- Reject a fresh bad-only download without leaving a header-only GRF.
- Quiet-install the Inno Setup installer into a disposable folder, verify
  launcher/config/guide presence, unsafe headless startup refusal, upgrade
  config preservation, and uninstall.

All test outputs should declare actual status. The CI artifact is retained
7 days and is NOT a signed release or a production patch channel.

## Known limits before live patching

- The full `LevelUpGT/RagnarokClient/client/base` is not checked into Git,
  so this installer is **patcher only**, not the full Ragnarok client.
- Exact 20220406 EXE SHA, DATA.INI order and protected loader compatibility
  require isolated game-clone verification; loader has not been proven.
- This release does **not** modify `ragnardata.grf`, `data.grf`,
  `tdata1.grf` or the real `DATA.INI`.
- Existing patches and GRF recovery backups must be validated on a disposable
  clone with rollback and actual readback before production feed approval.
- The public 2022 `b1glord/ragnarokclientpatch` feed remains unpublished.
- A source CI PASS is not native Windows gameplay smoke or map-warp PASS.
- Built QA installer needs signing/provenance review before public distribution.
- Default config resides beside the per-user installed binary and is not
  overwritten on setup upgrades. Game-directory choice is explicit.

## Development boundaries

- Source branch: `feat/windows-setup-patch-qa-20261025` with PR #13 stacked on
  visual theme PR #8. No merging, production release, tag or deployment.
- `cargo test -p beam-formats --lib`, `cargo test -p beam-core --lib`,
  `cargo build --release -p beam-patcher` and Windows Inno smoke must pass
  independently. Compile checks do not count as actual gameplay validation.
