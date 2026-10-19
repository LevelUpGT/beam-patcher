<!-- File Path: docs/CP949-GRF-BEAM-BYTE-PRESERVATION.md
Purpose: Evidence and safety gate for legacy CP949 GRF filename handling
Version: 0.1.0-candidate; no game writes, conversion or release -->

# Historical THOR CP949 filename blocker in Beam Patcher

Verified user-side 2026-10-19 isolated extraction: 26 enabled THOR
archives, 2,590 files (290,594,155 bytes) SHA verified, 12 collisions
within original GRF target, zero between distinct original GRF targets,
717 source GRF filenames with non-ASCII CP949 bytes; 1,250 loose files.
Original source-catalog SHA256:
4E6A62D67738214320CA1B276D076307629CB3956AED05E76C97CD9BFA0D02EA.

## Code-level failure mode

beam-formats/src/grf.rs reads GRF table filename bytes using
String::from_utf8_lossy(&filename_bytes).to_string() then serializes
entry.filename.as_bytes() on save. These are NOT byte-preserving for
CP949 input: replacement characters may be inserted and original byte
names lost. beam-formats/src/beam.rs likewise uses from_utf8_lossy
for source Beam filename decoding. Existing GRF copy-through is at risk.

## Acceptance criteria before any CP949 -> 2022 BEAM

1. Choose explicit source-client codepage, preserve exact name bytes
   through read/patch/save, and reject unrepresentable names.
2. Unit test CP949 and ASCII names, byte/name collisions, malicious
   paths, GRF 0x200 read/write/readback and original-name checksum.
3. Verify whole-file SHA256 and in-GRF decompressed payload after apply
   and power-loss/interrupt rollback in temporary directories.
4. Verify actual 20220406 EXE and DATA.INI priority-0 tdata1.grf.
5. Require individually approved reviewed manifest; no silent THOR
   retarget, no loose root files, no 2015 EXE/DLL migration.

Status: BLOCKED. Existing 623 ASCII-named GRF source files are only
2022-review candidates, not player-deployment-ready. This document
does not make runtime code changes or authorize any release.
