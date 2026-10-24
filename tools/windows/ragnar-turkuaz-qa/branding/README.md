<!--
Dosya Yolu: REPO_ROOT/tools/windows/ragnar-turkuaz-qa/branding/README.md
Amac: Onayli Concept C kurulum arka planinin kaynak/butunluk kontrati
Modul - Markdown
Version: 1.0.0
Aciklama: Opsiyonel PNG, hash pinli embed; varsayilan fallback korunur
Bagimli Oldugu Katman: View | Windows QA
-->

# Turkuaz Ragnarok Online — Concept C

Approved sample: third fantasy landscape image in the design review.
Place the original user-approved binary at `branding/concept-c-background.png`
before creating the new Windows QA artifact. It must have SHA-256:

`14b004262f8133cbd08cf784ba91ef2070886c185f9d2fbc7b8e32d432b7766b`

The CI embeds this PNG as Win32 RCDATA resource ID 102 **only if present**
and **only if the entire file matches the pinned hash**. The native
setup loads it read-only with GDI+ and preserves an offline gradient
fallback when absent. Never download arbitrary art or overwrite the
pinned identity from web/CDN at build or runtime. The image is a QA
visual asset, not a signing/antivirus verdict.

Source image transfer into the GitHub repository is separately required:
a ChatGPT sandbox image attachment is not a remotely fetchable CI URL.
The repo does not yet contain the binary image unless an authorized
maintainer uploads the exact vetted file.
