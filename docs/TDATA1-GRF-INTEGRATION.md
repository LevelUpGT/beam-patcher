<!--
File Path: docs/TDATA1-GRF-INTEGRATION.md
Purpose: Safe 20220406 GRF overlay and protected Beam target migration
Module - Markdown Documentation
Version: 0.1.0-candidate
Description: Read-only runtime preflight; NOT an installer, bootstrap or release
Dependency Layer: PatchCompatibilityGuard | DATA.INI | RagnarokClient PR #10/#9
-->

# Protected custom tdata1.grf overlay preflight

## Verified source behavior

- rAthena DATA.INI priority: slot 0 is highest, 0..9 supported:
  https://github.com/rathena/rathena/wiki/DATA.INI
- The historic RagnarokClient patches/legacy/DATA.INI.thor holds loose root data.ini, priorities 1..9 and no priority 0, plus an opaque [RagnaGuard] section. This is NOT proof of installed client configuration.
- BEAM v1 archives have internal GRF paths but no outer target GRF filename. Patcher uses patcher.target_grf.
- Historical sclientinfo.thor header targets ragnardata.grf, absent from archived DATA.INI. Blind replay into 20220406 is unsafe.
- Without an explicit guard, a missing configured GRF may be silently created and ignored by the client.

## New opt-in runtime check

Only when patcher.target_grf is tdata1.grf, require before downloader initialization and immediately before GRF writes:
1. Compatibility guard enabled, region=levelupgt, channel=custom (full EXE and patch manifest verification remains an independent existing guard).
2. The selected game directory DATA.INI is an existing regular non-symlink file with [Data] 0=tdata1.grf.
3. No duplicate slots, duplicate basenames, traversal or 10+ priorities.
4. Every named GRF file exists and is a regular non-symlink file.
5. Existing tdata1.grf has the required GRF header; deeper GRF validation still occurs when opened.

Other configured target GRFs remain unchanged. No actual client, GRF, THOR, DATA.INI, patch cache, patchlist or remote feed is modified by this read-only check.

## Planned activation sequence (NOT executed)

1. Identify the actual 20220406 executable and confirm it was patched with Enable Multiple GRFs. Confirm region and client SHA.
2. Back up actual installed client DATA.INI and GRFs. Historical ZIP/THOR DATA.INI is not guaranteed identical.
3. Create a valid standalone tdata1.grf in disposable staging and verify its content.
4. Stage DATA.INI inserting only priority 0 using RagnarokClient PR #10 data_ini_grf_layers.py. Preserve all [RagnaGuard] bytes and priorities 1..9.
5. In a disposable client, verify signed manifest/EXE, existing tdata1 and DATA.INI before passing through Beam.
6. Set Beam patcher.target_grf=tdata1.grf with configured compatibility guard; never use a mutable unprotected unsigned feed for player updates.
7. Apply Overlook or other approved GRF-only BEAM packages, verify final GRF entry SHA256, rollback on interrupted apply, test maps and server handshake.
8. Enable the related RagnarokServer Overlook addon only after the actual client/gameplay tests.

This guard requires a pre-created tdata1 GRF, so the installer/bootstrap must be separately implemented and accepted. No production player updater or release is authorized.
