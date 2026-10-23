// Dosya Yolu: tools/windows/ragnar-turkuaz-qa/RagnarSetupWizard.cpp
// Amac: Tek dosyali yerel Ragnar preview kurulumunu kullanici secenekleriyle baslatmak
// Modul - Windows native C++ Setup UI
// Version: 0.2.1-qa
// Aciklama: Embedded verified Velopack Setup, klasor secimi, kısayol tercihleri ve marka splash
// Bagimli Oldugu Katman: Velopack Setup | TurkuazInstaller preview QA build
// Note: This small user-facing wizard is NOT the full TurkuazInstaller Core runtime.

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <commctrl.h>
#include <shlobj.h>
#include <shobjidl.h>
#include <objbase.h>
#include <bcrypt.h>
#include <algorithm>
#include <array>
#include <cwchar>
#include <cwctype>
#include <string>
#include <vector>

#include "setup_hash.h"

#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "bcrypt.lib")
#pragma comment(lib, "user32.lib")

namespace {
constexpr int kSetupResourceId = 101;
constexpr wchar_t kProductTitle[] = L"Ragnar Classic Visual Preview";
constexpr wchar_t kShortcutFile[] = L"Ragnar Classic Visual Preview.lnk";
constexpr wchar_t kPackageExe[] = L"RagnarClassicPreview-Setup.exe";
constexpr wchar_t kVersion[] = L"v0.1.0 - QA";
constexpr wchar_t kInstalledLauncher[] = L"RagnarClassicPreview.exe";

struct Bytes { const BYTE* data; DWORD length; };

std::wstring Message(const wchar_t* reason) {
    return std::wstring(reason) + L"\n\nBu paket sadece gorunum testidir; oyunun dosyalari degistirilmez.";
}
void Alert(const std::wstring& text, UINT icon = MB_ICONERROR) {
    MessageBoxW(nullptr, text.c_str(), L"Ragnar Kurulum | Turkuaz QA", MB_OK | icon);
}

// TaskDialogIndirect is ordinal 345 in ComCtl32 v6. Statically importing it
// causes Windows 10 loader error 127/ordinal 345 before wWinMain when a v5
// comctl32 is selected. Resolve it at runtime and fail closed with a message.
using TaskDialogIndirectFn =
    HRESULT (WINAPI*)(const TASKDIALOGCONFIG*, int*, int*, BOOL*);

bool HasTaskDialogRuntime() {
    HMODULE library = LoadLibraryW(L"comctl32.dll");
    if (!library) return false;
    const bool found = GetProcAddress(library, "TaskDialogIndirect") != nullptr;
    FreeLibrary(library);
    return found;
}

HRESULT InvokeTaskDialog(const TASKDIALOGCONFIG* config, int* button,
                         int* radio, BOOL* checked) {
    HMODULE library = LoadLibraryW(L"comctl32.dll");
    if (!library) return HRESULT_FROM_WIN32(GetLastError());
    const auto function = reinterpret_cast<TaskDialogIndirectFn>(
        GetProcAddress(library, "TaskDialogIndirect"));
    if (!function) {
        FreeLibrary(library);
        return HRESULT_FROM_WIN32(ERROR_PROC_NOT_FOUND);
    }
    const HRESULT result = function(config, button, radio, checked);
    FreeLibrary(library);
    return result;
}

Bytes EmbeddedSetup() {
    HRSRC resource = FindResourceW(nullptr, MAKEINTRESOURCEW(kSetupResourceId), RT_RCDATA);
    if (!resource) return {nullptr, 0};
    const HGLOBAL loaded = LoadResource(nullptr, resource);
    if (!loaded) return {nullptr, 0};
    return {static_cast<const BYTE*>(LockResource(loaded)), SizeofResource(nullptr, resource)};
}

bool HashSha256(const BYTE* data, DWORD length, std::wstring& hex) {
    BCRYPT_ALG_HANDLE algorithm = nullptr;
    BCRYPT_HASH_HANDLE hash = nullptr;
    DWORD objSize = 0, digestSize = 0, returned = 0;
    bool ok = false;
    if (BCryptOpenAlgorithmProvider(&algorithm, BCRYPT_SHA256_ALGORITHM, nullptr, 0) < 0) return false;
    do {
        if (BCryptGetProperty(algorithm, BCRYPT_OBJECT_LENGTH, reinterpret_cast<PUCHAR>(&objSize),
            sizeof(objSize), &returned, 0) < 0) break;
        if (BCryptGetProperty(algorithm, BCRYPT_HASH_LENGTH, reinterpret_cast<PUCHAR>(&digestSize),
            sizeof(digestSize), &returned, 0) < 0 || digestSize != 32) break;
        std::vector<BYTE> object(objSize);
        std::array<BYTE, 32> digest{};
        if (BCryptCreateHash(algorithm, &hash, object.data(), objSize, nullptr, 0, 0) < 0) break;
        if (BCryptHashData(hash, const_cast<PUCHAR>(data), length, 0) < 0) break;
        if (BCryptFinishHash(hash, digest.data(), DWORD(digest.size()), 0) < 0) break;
        static constexpr wchar_t digits[] = L"0123456789abcdef";
        hex.clear();
        for (const BYTE value : digest) {
            hex.push_back(digits[value >> 4]);
            hex.push_back(digits[value & 15]);
        }
        ok = true;
    } while (false);
    if (hash) BCryptDestroyHash(hash);
    BCryptCloseAlgorithmProvider(algorithm, 0);
    return ok;
}

bool VerifyEmbeddedSetup() {
    const Bytes bytes = EmbeddedSetup();
    if (!bytes.data || bytes.length < 1024) return false;
    std::wstring actual;
    return HashSha256(bytes.data, bytes.length, actual) &&
           actual == std::wstring(RAGNAR_EMBEDDED_SETUP_SHA256);
}

int ChoosePage(const wchar_t* heading, const std::wstring& details,
               const std::vector<TASKDIALOG_BUTTON>& buttons) {
    TASKDIALOGCONFIG config{};
    config.cbSize = sizeof(config);
    config.hwndParent = nullptr;
    config.dwFlags = TDF_ALLOW_DIALOG_CANCELLATION | TDF_SIZE_TO_CONTENT;
    config.pszWindowTitle = L"Ragnar Ragnarok Online | Turkuaz Installer";
    config.pszMainInstruction = heading;
    config.pszContent = details.c_str();
    config.pszMainIcon = TD_INFORMATION_ICON;
    config.cButtons = UINT(buttons.size());
    config.pButtons = buttons.data();
    int clicked = IDCANCEL;
    const HRESULT hr = InvokeTaskDialog(&config, &clicked, nullptr, nullptr);
    return FAILED(hr) ? IDCANCEL : clicked;
}

std::wstring DefaultFolder() {
    PWSTR folder = nullptr;
    if (FAILED(SHGetKnownFolderPath(FOLDERID_LocalAppData, 0, nullptr, &folder)) || !folder)
        return {};
    std::wstring base = folder;
    CoTaskMemFree(folder);
    return base + L"\\Programs\\RagnarClassicTurkuazQA";
}

std::wstring BrowseFolder(const std::wstring& oldValue) {
    IFileOpenDialog* picker = nullptr;
    if (FAILED(CoCreateInstance(CLSID_FileOpenDialog, nullptr, CLSCTX_INPROC_SERVER,
                                IID_PPV_ARGS(&picker)))) return {};
    DWORD options = 0;
    picker->GetOptions(&options);
    picker->SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST);
    picker->SetTitle(L"Ragnar Classic icin kurulum klasoru secin");
    IShellItem* initial = nullptr;
    if (SUCCEEDED(SHCreateItemFromParsingName(oldValue.c_str(), nullptr, IID_PPV_ARGS(&initial)))) {
        picker->SetFolder(initial);
        initial->Release();
    }
    std::wstring chosen;
    if (SUCCEEDED(picker->Show(nullptr))) {
        IShellItem* item = nullptr;
        if (SUCCEEDED(picker->GetResult(&item)) && item) {
            PWSTR value = nullptr;
            if (SUCCEEDED(item->GetDisplayName(SIGDN_FILESYSPATH, &value)) && value) {
                chosen = value;
                CoTaskMemFree(value);
            }
            item->Release();
        }
    }
    picker->Release();
    return chosen;
}

bool SafeTarget(std::wstring& folder) {
    if (folder.empty() || folder.size() >= MAX_PATH - 60) return false;
    wchar_t full[MAX_PATH]{};
    const DWORD n = GetFullPathNameW(folder.c_str(), MAX_PATH, full, nullptr);
    if (n == 0 || n >= MAX_PATH) return false;
    folder = full;
    if (folder.find_first_of(L"\"<>|?*") != std::wstring::npos)
        return false;
    if (folder.size() < 5 || folder[1] != L':' || folder[2] != L'\\') return false;
    const std::wstring lower = [&] {
        std::wstring copy = folder;
        std::transform(copy.begin(), copy.end(), copy.begin(),
                       [](wchar_t c){ return std::towlower(c); });
        return copy;
    }();
    wchar_t windowsPath[MAX_PATH]{};
    if (GetWindowsDirectoryW(windowsPath, MAX_PATH)) {
        std::wstring win = windowsPath;
        std::transform(win.begin(), win.end(), win.begin(), [](wchar_t c){ return std::towlower(c); });
        if (lower == win || lower.rfind(win + L"\\", 0) == 0) return false;
    }
    if (folder.back() == L'\\') return false; // Reject drive roots, keep installs in own subfolder.
    return true;
}

std::wstring MakeTempDirectory() {
    wchar_t buffer[MAX_PATH]{};
    const DWORD count = GetTempPathW(MAX_PATH, buffer);
    if (count == 0 || count >= MAX_PATH) return {};
    GUID id{};
    if (FAILED(CoCreateGuid(&id))) return {};
    wchar_t guid[64]{};
    if (!StringFromGUID2(id, guid, 64)) return {};
    std::wstring folder = std::wstring(buffer) + L"RagnarTurkuazQa-" + guid;
    if (!CreateDirectoryW(folder.c_str(), nullptr)) return {};
    return folder;
}

bool ExtractSetup(const std::wstring& target) {
    const Bytes bytes = EmbeddedSetup();
    if (!bytes.data || !VerifyEmbeddedSetup()) return false;
    HANDLE file = CreateFileW(target.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW,
                              FILE_ATTRIBUTE_TEMPORARY, nullptr);
    if (file == INVALID_HANDLE_VALUE) return false;
    DWORD written = 0;
    const bool ok = WriteFile(file, bytes.data, bytes.length, &written, nullptr) != 0 &&
                    written == bytes.length && FlushFileBuffers(file) != 0;
    CloseHandle(file);
    if (!ok) DeleteFileW(target.c_str());
    return ok;
}

bool RunSetup(const std::wstring& setup, const std::wstring& target, DWORD& exitCode) {
    std::wstring arguments = L"\"" + setup + L"\" --installto \"" + target + L"\"";
    std::vector<wchar_t> command(arguments.begin(), arguments.end());
    command.push_back(0);
    STARTUPINFOW startup{sizeof(startup)};
    PROCESS_INFORMATION proc{};
    if (!CreateProcessW(setup.c_str(), command.data(), nullptr, nullptr, FALSE, 0,
                        nullptr, nullptr, &startup, &proc)) return false;
    WaitForSingleObject(proc.hProcess, INFINITE);
    const bool gotExit = GetExitCodeProcess(proc.hProcess, &exitCode) != 0;
    CloseHandle(proc.hThread);
    CloseHandle(proc.hProcess);
    return gotExit;
}

std::wstring KnownFolder(REFKNOWNFOLDERID key) {
    PWSTR path = nullptr;
    if (FAILED(SHGetKnownFolderPath(key, 0, nullptr, &path)) || !path) return {};
    std::wstring result = path;
    CoTaskMemFree(path);
    return result;
}
std::wstring Lower(std::wstring source) {
    std::transform(source.begin(), source.end(), source.begin(), [](wchar_t c){ return std::towlower(c); });
    return source;
}

// Never remove a shortcut unless it actually targets THIS newly installed preview.
// Velopack creates both by default; the wizard removes declined destinations.
void RemoveDeclinedShortcut(const std::wstring& directory, const std::wstring& root) {
    if (directory.empty()) return;
    std::wstring path = directory + L"\\" + kShortcutFile;
    IShellLinkW* link = nullptr;
    if (FAILED(CoCreateInstance(CLSID_ShellLink, nullptr, CLSCTX_INPROC_SERVER,
                                IID_PPV_ARGS(&link)))) return;
    IPersistFile* persisted = nullptr;
    if (SUCCEEDED(link->QueryInterface(IID_PPV_ARGS(&persisted)))) {
        if (SUCCEEDED(persisted->Load(path.c_str(), STGM_READ))) {
            wchar_t destination[MAX_PATH]{};
            WIN32_FIND_DATAW info{};
            if (SUCCEEDED(link->GetPath(destination, MAX_PATH, &info, SLGP_RAWPATH))) {
                const auto target = Lower(destination);
                const auto expectedRoot = Lower(root + L"\\");
                if (target.rfind(expectedRoot, 0) == 0 &&
                    target.size() >= wcslen(kInstalledLauncher) &&
                    target.compare(target.size()-wcslen(kInstalledLauncher),
                                   wcslen(kInstalledLauncher), Lower(kInstalledLauncher)) == 0) {
                    DeleteFileW(path.c_str());
                }
            }
        }
        persisted->Release();
    }
    link->Release();
}

int InstallInteractive() {
    if (!VerifyEmbeddedSetup()) {
        Alert(Message(L"Kurulum içeriğinin SHA-256 doğrulaması başarısız.")); return 4;
    }
    const std::vector<TASKDIALOG_BUTTON> welcome{{100, L"Devam et"}, {IDCANCEL, L"İptal"}};
    if (ChoosePage(L"RAGNAR RAGNAROK ONLINE",
          std::wstring(L"") + kVersion +
          L"\nTurkuaz QA | Klasik istemci arayuzu\n\n"
          L"Bu paket sunucuya baglanmaz, patch uygulamaz ve mevcut oyun klasorune yazmaz.\n"
          L"Kurulum yeri ve kisayollari secerek ilerleyebilirsiniz.", welcome) != 100) return 0;

    std::wstring folder = DefaultFolder();
    if (!SafeTarget(folder)) { Alert(L"Guvenli varsayilan klasor alinamadi."); return 4; }
    for (;;) {
        const std::vector<TASKDIALOG_BUTTON> location{{100, L"Bu klasöre kur"}, {101, L"Başka klasör seç"}, {IDCANCEL, L"Iptal"}};
        const int answer = ChoosePage(L"Kurulum konumu", folder + L"\n\nKurulum sadece bu onizlemeyi etkiler.", location);
        if (answer == 100) break;
        if (answer != 101) return 0;
        const auto picked = BrowseFolder(folder);
        if (picked.empty()) continue;
        std::wstring proposed = picked + L"\\RagnarClassicTurkuazQA";
        if (!SafeTarget(proposed)) { Alert(L"Gecersiz kurulum konumu."); continue; }
        folder = proposed;
    }

    const TASKDIALOG_BUTTON radios[] = {
        {10, L"Masaüstü ve Başlat menüsü"},
        {11, L"Yalnızca Başlat menüsü"},
        {12, L"Yalnızca Masaüstü"},
        {13, L"Kısayol oluşturma"}
    };
    TASKDIALOGCONFIG choices{};
    choices.cbSize = sizeof(choices);
    choices.pszWindowTitle = L"Ragnar | Kisayol tercihleri";
    choices.pszMainInstruction = L"Kısayolları nereye ekleyelim?";
    choices.pszContent = L"Seçiminiz kurulum tamamlandığında uygulanır.";
    choices.dwFlags = TDF_ALLOW_DIALOG_CANCELLATION | TDF_SIZE_TO_CONTENT;
    choices.cRadioButtons = ARRAYSIZE(radios);
    choices.pRadioButtons = radios;
    choices.nDefaultRadioButton = 10;
    choices.dwCommonButtons = TDCBF_OK_BUTTON | TDCBF_CANCEL_BUTTON;
    int answer = IDCANCEL, option = 10;
    if (FAILED(InvokeTaskDialog(&choices, &answer, &option, nullptr)) || answer != IDOK) return 0;

    const std::vector<TASKDIALOG_BUTTON> install{{100, L"Kur"}, {IDCANCEL, L"Iptal"}};
    if (ChoosePage(L"Kuruluma hazır", folder +
             L"\n\nSurum: v0.1.0 Preview\nYonetici yetkisi gerekmez.\n"
             L"Yükleme ekranı kurulum sırasında görüntülenir.", install) != 100) return 0;

    const auto tempRoot = MakeTempDirectory();
    if (tempRoot.empty()) { Alert(L"Gecici klasor olusturulamadi."); return 4; }
    const std::wstring setupFile = tempRoot + L"\\" + kPackageExe;
    DWORD exitCode = 0;
    const bool extracted = ExtractSetup(setupFile);
    const bool ran = extracted && RunSetup(setupFile, folder, exitCode);
    DeleteFileW(setupFile.c_str());
    RemoveDirectoryW(tempRoot.c_str());
    if (!ran || exitCode != 0) {
        Alert(L"Kurulum tamamlanamadı. Hata kodu: " + std::to_wstring(exitCode));
        return 4;
    }
    if (option == 11 || option == 13) RemoveDeclinedShortcut(KnownFolder(FOLDERID_Desktop), folder);
    if (option == 12 || option == 13) RemoveDeclinedShortcut(KnownFolder(FOLDERID_Programs), folder);
    Alert(L"Ragnar Classic önizleme kuruldu.\n\nGerçek oyun dosyaları değiştirilmedi.", MB_ICONINFORMATION);
    return 0;
}
} // namespace

int WINAPI wWinMain(HINSTANCE, HINSTANCE, LPWSTR commandLine, int) {
    if (commandLine && wcscmp(commandLine, L"--verify-payload") == 0)
        return VerifyEmbeddedSetup() ? 0 : 4;
    if (commandLine && wcscmp(commandLine, L"--verify-dialog-runtime") == 0)
        return HasTaskDialogRuntime() ? 0 : 4;
    if (commandLine && *commandLine) {
        Alert(L"Desteklenmeyen komut satırı seçeneği.");
        return 2;
    }
    const HRESULT init = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    if (FAILED(init)) { Alert(L"Windows iletişim sistemi hazırlanamadı."); return 4; }
    if (!HasTaskDialogRuntime()) {
        Alert(L"Windows Common Controls v6 etkinleştirilemedi. "
              L"Kurulum güvenli şekilde durduruldu.");
        CoUninitialize();
        return 4;
    }
    const int code = InstallInteractive();
    CoUninitialize();
    return code;
}
