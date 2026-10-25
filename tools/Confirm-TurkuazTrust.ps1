# Dosya Yolu: REPO_ROOT/tools/Confirm-TurkuazTrust.ps1
# Amac: Turkuaz primary installer manifest CMS, external pin ve Authenticode gercek dogrulama
# Modul - PowerShell / Protected Windows signing gate
# Version: 1.0.0-candidate
# Aciklama: PASS raporu yalniz ilave test; bu kod gercek byte imza zincirini kontrol eder
# Bagimli Oldugu Katman: tools/verify_installer_pair.py | TurkuazInstaller/docs/MANIFEST_TRUST.md

[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PrimaryDirectory,
    [Parameter(Mandatory)][string]$PublisherSubject,
    [Parameter(Mandatory)][string]$CertificateSha256,
    [string]$PackageId = 'levelupgt.beampatcher.qa'
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Primary = (Resolve-Path -LiteralPath $PrimaryDirectory).Path
$CertPin = $CertificateSha256.Trim().ToLowerInvariant()
if ($CertPin -notmatch '^[a-f0-9]{64}$' -or [string]::IsNullOrWhiteSpace($PublisherSubject)) {
    throw 'BURAK-INS-001: independent signer subject and 64-hex SHA256 pin required'
}
$BinPaths = @(
    'TurkuazInstaller.Bootstrapper.exe',
    'app/TurkuazInstaller.WinUI.exe',
    'cli/TurkuazInstaller.Cli.exe'
)
foreach ($Rel in $BinPaths) {
    $Binary = Join-Path $Primary $Rel
    if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) { throw "Missing signed Turkuaz executable: $Rel" }
    $Sig = Get-AuthenticodeSignature -LiteralPath $Binary
    if ($Sig.Status -ne 'Valid' -or $null -eq $Sig.SignerCertificate) {
        throw "Turkuaz binary Authenticode not trusted: $Rel, $($Sig.Status)"
    }
    $Sign = $Sig.SignerCertificate
    $Hash = [Convert]::ToHexString(
        [Security.Cryptography.SHA256]::HashData($Sign.RawData)
    ).ToLowerInvariant()
    if ($Sign.Subject -cne $PublisherSubject -or $Hash -cne $CertPin) {
        throw "Turkuaz publisher/certificate pin mismatch: $Rel"
    }
}
$Manifest = Join-Path $Primary 'installer-manifest.yml'
$Signature = Join-Path $Primary 'installer-manifest.yml.p7s'
$Trust = Join-Path $Primary 'manifest-trust.yml'
foreach ($Path in @($Manifest,$Signature,$Trust)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing signed manifest/trust input: $Path"
    }
}

# Check the detached CMS signature against the exact bytes saved on disk.
Add-Type -AssemblyName 'System.Security.Cryptography.Pkcs'
$Content = [Security.Cryptography.Pkcs.ContentInfo]::new([IO.File]::ReadAllBytes($Manifest))
$Cms = [Security.Cryptography.Pkcs.SignedCms]::new($Content, $true)
$Cms.Decode([IO.File]::ReadAllBytes($Signature))
$Cms.CheckSignature($true)
if ($Cms.SignerInfos.Count -ne 1 -or $null -eq $Cms.SignerInfos[0].Certificate) {
    throw 'Expected one detached CMS manifest signer'
}
$Signer = $Cms.SignerInfos[0].Certificate
$SignerSha = [Convert]::ToHexString(
    [Security.Cryptography.SHA256]::HashData($Signer.RawData)
).ToLowerInvariant()
if ($Signer.Subject -cne $PublisherSubject -or $SignerSha -cne $CertPin) {
    throw 'Detached CMS manifest signer does not match independent certificate pin'
}
$TrustText = Get-Content -LiteralPath $Trust -Raw -Encoding utf8
if (-not $TrustText.Contains("id: $PackageId") -or
    -not $TrustText.Contains("certificate_sha256: $CertPin") -or
    -not $TrustText.Contains("publisher_subject: $PublisherSubject")) {
    throw 'External trust store does not contain the exact package and publisher pin'
}
$ManifestText = Get-Content -LiteralPath $Manifest -Raw -Encoding utf8
if (-not $ManifestText.Contains("id: $PackageId") -or
    -not $ManifestText.Contains('version: 1.0.1')) {
    throw 'Signed manifest does not identify the pinned Beam QA package/version'
}
$Setup = Join-Path $Primary 'Beam-Velopack-Setup.exe'
$Package = Join-Path $Primary 'Beam-full.nupkg'
$HashOfSetup = (Get-FileHash -LiteralPath $Setup -Algorithm SHA256).Hash.ToLowerInvariant()
$HashOfNupkg = (Get-FileHash -LiteralPath $Package -Algorithm SHA256).Hash.ToLowerInvariant()
# At least one exact artifact hash must be bound by the signed manifest.
if (-not $ManifestText.Contains($HashOfSetup) -and -not $ManifestText.Contains($HashOfNupkg)) {
    throw 'Signed Turkuaz manifest does not bind actual Velopack artifact hash'
}
Write-Output 'BURAK-INS-001_REAL_SIGNATURE_AND_EXTERNAL_PIN_OK'
