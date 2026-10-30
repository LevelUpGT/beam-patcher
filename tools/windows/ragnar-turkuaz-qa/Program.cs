// Dosya Yolu: tools/windows/ragnar-turkuaz-qa/Program.cs
// Amac: Isolated Ragnar TurkuazInstaller QA signed local manifest provisioning
// Modul - .NET 10 CSharp Tool
// Version: 0.1.0-qa
// Aciklama: Creates an ephemeral local-only CMS test signature; refuses to overwrite real trust or UI profile.
// Bagimli Oldugu Katman: Tool | TurkuazInstaller Community signed manifest and branding contracts

using System.Security.Cryptography;
using System.Security.Cryptography.Pkcs;
using System.Security.Cryptography.X509Certificates;
using System.Text;
using System.Text.Json;

internal static class Program
{
    private const string PackageId = "levelupgt.ragnar-classic-preview";
    private const string OwnerMarker = "# ragnar-turkuaz-qa generated local test only";
    private const string PayloadName = "RagnarClassicPreview-Setup.exe";
    private const string ManifestName = "installer-manifest.yml";
    private const string ProfileName = "ui-profile.yml";

    private static int Main(string[] args)
    {
        Console.OutputEncoding = Encoding.UTF8;
        if (args.Length != 2 || (args[0] != "prepare" && args[0] != "cleanup"))
        {
            Console.Error.WriteLine("Usage: Ragnar.QaProvisioner.exe prepare|cleanup <unpacked-artifact-directory>");
            return 2;
        }

        try
        {
            var root = Path.GetFullPath(args[1]);
            var qa = Path.Combine(root, "qa");
            var configRoot = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "TurkuazInstaller",
                "config");
            var trustPath = Path.Combine(configRoot, "manifest-trust.yml");
            var profilePath = Path.Combine(configRoot, ProfileName);
            var manifestPath = Path.Combine(qa, ManifestName);
            var signaturePath = manifestPath + ".p7s";

            if (args[0] == "cleanup")
            {
                DeleteOnlyOurConfig(trustPath);
                DeleteOnlyOurConfig(profilePath);
                Console.WriteLine("RAGNAR_TURKUAZ_QA_OWNED_CONFIG_CLEANUP_OK");
                return 0;
            }

            var payload = Path.Combine(root, "payload", PayloadName);
            var profile = Path.Combine(qa, ProfileName);
            if (!File.Exists(payload) || !File.Exists(profile) ||
                !File.Exists(Path.Combine(root, "cli", "TurkuazInstaller.Cli.exe")))
            {
                throw new FileNotFoundException("Incomplete bundle. Extract the full artifact ZIP.");
            }

            // Never modify an existing user's global trust policy or branding.
            if (File.Exists(trustPath) || File.Exists(profilePath))
            {
                throw new InvalidOperationException(
                    "Existing TurkuazInstaller user trust/UI config detected. " +
                    "Use a disposable Windows QA user/VM; existing settings will not be overwritten.");
            }

            if (File.Exists(manifestPath) || File.Exists(signaturePath))
            {
                throw new InvalidOperationException(
                    "Manifest already exists. No implicit overwrite or re-signing is allowed.");
            }

            var payloadInfo = new FileInfo(payload);
            if (payloadInfo.Length < 1 || (payloadInfo.Attributes & FileAttributes.ReparsePoint) != 0)
            {
                throw new InvalidDataException("Payload is empty or a reparse point.");
            }

            using var stream = File.OpenRead(payload);
            var sha256 = Convert.ToHexString(SHA256.HashData(stream)).ToLowerInvariant();
            // JSON quoted strings are valid YAML scalar strings, including Unicode/windows file URI.
            var fileUri = JsonSerializer.Serialize(new Uri(payloadInfo.FullName).AbsoluteUri);
            var manifest = $"""
                schema_version: 1
                package:
                  id: {PackageId}
                  version: 0.1.0
                  channel: stable
                artifact:
                  uri: {fileUri}
                  sha256: {sha256}
                  size_bytes: {payloadInfo.Length}
                install:
                  mode: full
                  target: "%LOCALAPPDATA%/Programs/RagnarClassicTurkuazQA"
                  prerequisites:
                    - id: windows-build
                      version: ">=17763"
                    - id: architecture
                      version: "x64"
                  preserve_paths:
                    - "UserData"
                rollback:
                  supported: false
                """;
            var manifestBytes = new UTF8Encoding(false).GetBytes(manifest + "\n");
            using var rsa = RSA.Create(3072);
            var request = new CertificateRequest(
                "CN=Ragnar Classic Turkuaz QA Local Test",
                rsa,
                HashAlgorithmName.SHA256,
                RSASignaturePadding.Pkcs1);
            var now = DateTimeOffset.UtcNow;
            using var cert = request.CreateSelfSigned(now.AddMinutes(-5), now.AddDays(3));
            var signedCms = new SignedCms(new ContentInfo(manifestBytes), detached: true);
            var signer = new CmsSigner(SubjectIdentifierType.IssuerAndSerialNumber, cert)
            {
                IncludeOption = X509IncludeOption.EndCertOnly,
                DigestAlgorithm = new Oid("2.16.840.1.101.3.4.2.1")
            };
            signedCms.ComputeSignature(signer);
            var signatureBytes = signedCms.Encode();

            // First verify the signature locally; no private key is persisted or exported.
            var verify = new SignedCms(new ContentInfo(manifestBytes), detached: true);
            verify.Decode(signatureBytes);
            verify.CheckSignature(verifySignatureOnly: true);

            var pin = cert.GetCertHashString(HashAlgorithmName.SHA256).ToLowerInvariant();
            var trust = $"{OwnerMarker}\nschema_version: 1\npackages:\n" +
                        $"  - id: {PackageId}\n" +
                        $"    publisher_subject: {JsonSerializer.Serialize(cert.Subject)}\n" +
                        $"    certificate_sha256: {pin}\n";
            var brandBytes = File.ReadAllBytes(profile);
            if (!Encoding.UTF8.GetString(brandBytes).Contains("ragnar-turkuaz-qa: managed local preview branding",
                    StringComparison.Ordinal))
            {
                throw new InvalidDataException("QA branding ownership marker missing.");
            }

            Directory.CreateDirectory(configRoot);
            // Track only paths created by this invocation; never delete someone else's file.
            var created = new List<string>();
            try
            {
                WriteNew(manifestPath, manifestBytes); created.Add(manifestPath);
                WriteNew(signaturePath, signatureBytes); created.Add(signaturePath);
                WriteNew(trustPath, new UTF8Encoding(false).GetBytes(trust)); created.Add(trustPath);
                WriteNew(profilePath, brandBytes); created.Add(profilePath);
            }
            catch
            {
                foreach (var createdPath in created.AsEnumerable().Reverse())
                {
                    if (File.Exists(createdPath)) File.Delete(createdPath);
                }
                throw;
            }
            Console.WriteLine("RAGNAR_TURKUAZ_QA_SIGNED_LOCAL_MANIFEST_OK");
            Console.WriteLine("File: " + manifestPath);
            Console.WriteLine("Artifact SHA256: " + sha256);
            Console.WriteLine("QA signer SHA256: " + pin);
            Console.WriteLine("WARNING: Self-signed ephemeral local QA trust; NOT production publisher identity.");
            return 0;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine("RAGNAR_TURKUAZ_QA_FAILED: " + error.Message);
            return 4;
        }
    }

    private static void WriteNew(string path, byte[] data)
    {
        using var output = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None);
        output.Write(data);
        output.Flush(flushToDisk: true);
    }

    private static void DeleteOnlyOurConfig(string path)
    {
        if (!File.Exists(path)) return;
        var text = File.ReadAllText(path);
        if (!text.Contains(OwnerMarker, StringComparison.Ordinal) &&
            !text.Contains("# ragnar-turkuaz-qa: managed local preview branding", StringComparison.Ordinal))
        {
            throw new InvalidOperationException("Refusing to delete a non-QA config file: " + path);
        }
        File.Delete(path);
    }
}
