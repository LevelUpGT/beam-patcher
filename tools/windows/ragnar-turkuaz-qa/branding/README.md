<!--
Dosya Yolu: tools/windows/ragnar-turkuaz-qa/branding/README.md
Amac: Kurulum icin onayli coklu Ragnarok tema arka planlarini tanimlar
Modul - Markdown
Version: 2.0.0
Aciklama: Uc hash-sabitli PNG ve kaynak dogrulama; bos/eksik asset raporlama
Bagimli Oldugu Katman: Windows View | Build | Security
-->

# Turkuaz Ragnarok Online — Three-stage approved artwork

| Resource ID | File | Page | SHA-256 |
| --- | --- | --- | --- |
| 102 | `concept-c-background.png` | Welcome (approved original Concept C) | `14b004262f8133cbd08cf784ba91ef2070886c185f9d2fbc7b8e32d432b7766b` |
| 103 | `sunrise-background.png` | Options / Complete | `c9b019b93709a42f0b841a475f1d1f06f249fab10d16b09bba73288b05933bfc` |
| 104 | `twilight-background.png` | Installing / Failed | `f907a41de5142c47e79a25a74ad140b78571463c54fe063dd099376a4403d739` |

The original **approved artwork** must be copied locally under the exact
filenames, within this folder, before building a screenshot QA package.
The manifest `backgrounds.json` is the source of expected file identity.
`validate_artwork.py` validates all expected hashes, PNG IHDR dimensions
and fixed resource IDs. It writes `artwork-identity.json` and an
`artwork.rc` snippet used to embed resource IDs 102–104.

**All-or-none:** if no artwork binaries are present, the wizard retains the
legacy gradient and QA metadata states `gradient-only`. Any partial set,
wrong digest, symlink or unexpected dimensions fails packaging; a CI success
on `gradient-only` is NOT proof of the new backgrounds rendering.

For all three approved binaries present, build metadata states
`approved-artwork`; the native runtime verifies the three images can be
decoded and `--verify-artwork-present` checks them. Actual Windows
100% / 125% / 150% DPI screenshots remain a manual acceptance step.

No runtime network wallpaper downloads, unexpected GUI launch, editing GRF,
change to packetver, local trust policy, AV exemptions or production signing.

**GitHub source limitation:** the ChatGPT-generated binary images have not
been uploaded to this repository through the GitHub connector yet.
The source code and visual test bundle are separate; run on the pinned
asset files before claiming a reskinned Windows setup EXE.
