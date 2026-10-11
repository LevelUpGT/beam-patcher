<!--
📄 Dosya Yolu: /LevelUpGT/beam-patcher/docs/BEAM-PRIVATE-REPO-ACCESS.md
📌 Amac: RagnarokClient gizli GitHub deposu icin salt-okunur yetkilendirme sinirlarini tanimlamak
📌 Modul - Markdown
Version: 1.0.0
Aciklama: GitHub App broker tasarimi ve kimlik bilgisi guvenligi
Bagimli Oldugu Katman: Controller | Service | Tool | View
-->

# BEAM - Private RagnarokClient Branch Discovery

## Verified repository state

- Target: `LevelUpGT/RagnarokClient` (private).
- ChatGPT GitHub connector has read permission for this repository. This is **not** desktop-app authentication.
- The desktop branch inspector currently rejects the private source without authorized broker access.

## Required production architecture

1. A server-side GitHub App installation is explicitly authorized for **only** `LevelUpGT/RagnarokClient` with read-only repository metadata/contents permission where needed.
2. Private GitHub App credentials and short-lived installation tokens remain **server-side** in a managed secret store; they must never appear in a desktop EXE, web UI, log, PR artifact, or plaintext settings.
3. BEAM users authenticate to an HTTPS branch metadata broker. Broker verifies session authorization before contacting GitHub and limits repository access by a fixed allowlist, not user-supplied URLs.
4. Broker returns only branch name, full commit SHA and protection flag. It must enforce response-size, time and rate limits. No source archives, patch payloads or installation feeds are returned.
5. HTTP failures and missing authorization are explicit and fail closed. Requests with malformed branch names, unsigned update manifests or redirect-to-untrusted-origin are refused.
6. Audit the broker's access while excluding credentials from telemetry. Revoke an installation immediately on permission loss.
7. Desktop profile selection stays independent of repository branch selection. Neither authorizes download, installation or runtime execution.

## Release blockers

- No authorized broker endpoint is configured in BEAM.
- No desktop user-to-broker authentication session exists.
- GitHub connection in ChatGPT cannot be reused as a desktop credential.
- Signed manifest trust roots, key rotation, actual file integrity and rollback are separate release gates.

**Security posture:** keep the private repo unavailable in the preview until this flow is provisioned and independently tested.
