# BEAM THOR safety — phase A

File Path: /docs/THOR-SAFETY-TEST-PLAN.md
Purpose: Explain forked BEAM's initial THOR correctness and safety tests
Module: ClientDistribution — Markdown
Version: 1.1.0
Description: P0 parsing plus Phase B experimental loose-file transactions and rollback tests
Dependency Layer: Tool

Source: MIT-licensed https://github.com/beamguides/beam-patcher; original LICENSE retained.
Our fork: https://github.com/b1glord/beam-patcher. Upstream baseline pinned to feed97887090d121f796bc1b941390e28b7a2da5.

## Completed in this phase (subject to CI)

1. Preserve the original THOR header \`use_grf_merging\` and \`target_grf_name\` instead of discarding it.
2. Refuse corrupted or truncated compressed payloads; do not generate empty replacement files on decode errors.
3. Bound compressed-file expansion (256 MiB per entry), THOR table expansion (16 MiB), and total file count (100,000). Validate size and offset references before decoding and enforce the declared number of entries.
4. Validate internal paths against \`../\`, drive-absolute paths, UNC prefixes, unsafe Windows device filenames and case-folded duplicate names. Reject unsafe or non-UTF-8 THOR filenames; a future legacy codepage adapter may be needed for particular 2015 archives.
5. Round-trip writer metadata and prevent untrusted target GRF names from escaping the intended data archive.
6. **Fail closed:** BEAM currently installs archives into GRF; when \`use_grf_merging=false\`, reject the THOR without modifying the configured GRF. Also reject a THOR targeting a different GRF. That means the current ROenglishRE Hybrid THOR is NOT yet supported on BEAM.

## Phase B — experimental loose-file THOR writer

- `beam-core/src/patcher.rs` now routes **GRF-free THOR** archives to a loose-client file writer.
- All patch downloads, SHA checks and THOR parse checks complete before any client write. Files are staged and backed up on the game's volume before replacing originals.
- Loose-client packages in the same batch are applied in order, with later THOR entries winning. The patcher records completed patches only after a successful batch.
- Bad paths, unsafe Windows device names, symlinked ancestors and case-ambiguous duplicates are refused. Staging and best-effort rollback preserve previous files if an ordinary file operation fails.
- **Mixed GRF+loose THOR batches are intentionally rejected**; they need a joint persistent transaction journal and ordering integration tests.
- **A sudden power failure or forced process termination during replacement is not fully recoverable yet.** Persistent journal/recovery is future P0 work. Do not use on production clients.
- No 2015 historical patch state, Thor Patcher config, rPatchur feed or game EXE has been modified.

CI exercises `beam-formats` and `beam-core` parser and transaction tests. It is NOT equivalent to a Windows GRF + client acceptance test.

## Acceptance checklist

- [ ] GitHub Actions \`cargo test -p beam-formats --lib\`: PASS
- [ ] GitHub Actions \`cargo test -p beam-core --lib\`: PASS
- [ ] GitHub Actions \`cargo check -p beam-core\`: PASS
- [ ] Real Windows 10 x64 build on FURY: NOT_RUN
- [ ] GRF vs loose file integration on a disposable client: NOT_RUN
- [ ] 2015 legacy THOR and 2022 Hybrid target file compatibility: NOT_RUN

**Do not distribute this branch as a production patcher.** Thor Patcher (old 2015) and rPatchur (new 2015 and 2022 Hybrid) remain unchanged.
