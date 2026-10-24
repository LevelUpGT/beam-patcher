// Dosya Yolu: tools/windows/ragnar-turkuaz-qa/RagnarSetupWindow.cpp
// Amac: Adobe tarzi tek ekran Ragnar kurulum akisini tasiyan native View
// Modul - Win32 Windows UI C++
// Version: 0.3.2-qa
// Aciklama: Splash is first; options and progress stay in one window; no extra dialogs
// Bagimli Oldugu Katman: RagnarSetupWizard.cpp adapter | Windows UI / GDI

#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <shlobj.h>
#include <wingdi.h>
#include <algorithm>
#include <memory>
#include <string>
#include <thread>

#pragma comment(lib, "user32.lib")
#pragma comment(lib, "gdi32.lib")
#pragma comment(lib, "msimg32.lib")
#pragma comment(lib, "ole32.lib")

bool RagnarVerify();
bool RagnarValidTarget(std::wstring&);
std::wstring RagnarDefaultTarget();
std::wstring RagnarTempFolder();
bool RagnarExtractSetup(const std::wstring&);
bool RagnarSilentInstall(const std::wstring&, const std::wstring&, DWORD&, std::wstring&);
bool RagnarCreateShortcut(bool, const std::wstring&);
bool RagnarOpenInstalled(const std::wstring&, DWORD&);
bool RagnarDialogAvailable();

namespace {
// Size the window for footer, option controls, and Windows non-client borders.
constexpr int W = 810, H = 640;
constexpr int kGo = 2001, kBack = 2002, kBrowse = 2003;
constexpr int kPath = 2004, kDesktop = 2005, kStart = 2006;
constexpr UINT kDone = WM_APP + 15;
constexpr UINT_PTR kPulse = 12;
constexpr wchar_t kTitle[] = L"Ragnar Ragnarok Online — Kurulum";
constexpr wchar_t kSetupName[] = L"RagnarClassicPreview-Setup.exe";
enum class Page { Welcome, Options, Installing, Complete, Failed };
struct Result { bool success = false; std::wstring message; };
struct State {
    HWND hwnd = nullptr, go = nullptr, back = nullptr, browse = nullptr;
    HWND folder = nullptr, desktop = nullptr, start = nullptr;
    HFONT heading = nullptr, body = nullptr, smallFont = nullptr;
    HBRUSH formBackground = nullptr;
    Page page = Page::Welcome;
    int pulse = 0;
    std::wstring destination, error;
} state;

void Show(HWND h, bool yes) { ShowWindow(h, yes ? SW_SHOW : SW_HIDE); }
void Caption(HWND h, const wchar_t* text) { SetWindowTextW(h, text); }
void Redraw() { InvalidateRect(state.hwnd, nullptr, FALSE); }
void SetPage(Page page) {
    state.page = page;
    bool choosing = page == Page::Options;
    Show(state.folder, choosing);
    Show(state.browse, choosing);
    Show(state.desktop, choosing);
    Show(state.start, choosing);
    Show(state.go, page != Page::Installing);
    Show(state.back, choosing || page == Page::Complete || page == Page::Failed);
    if (page == Page::Welcome) Caption(state.go, L"DEVAM ET  →");
    if (page == Page::Options) Caption(state.go, L"KURULUMU BAŞLAT");
    if (page == Page::Complete) Caption(state.go, L"RAGNAR'I AÇ");
    if (page == Page::Failed) Caption(state.go, L"TEKRAR DENE");
    Caption(state.back, page == Page::Options ? L"GERİ" : L"KAPAT");
    if (page == Page::Installing) SetTimer(state.hwnd, kPulse, 80, nullptr);
    else KillTimer(state.hwnd, kPulse);
    Redraw();
}
void Print(HDC dc, HFONT font, COLORREF color, const wchar_t* value,
           int x, int y, int width, int height, UINT flags = DT_NOPREFIX) {
    auto before = SelectObject(dc, font);
    SetTextColor(dc, color);
    SetBkMode(dc, TRANSPARENT);
    RECT rect{x,y,x+width,y+height};
    DrawTextW(dc, value, -1, &rect, flags);
    SelectObject(dc, before);
}
void Solid(HDC dc, RECT rect, COLORREF color) {
    HBRUSH brush = CreateSolidBrush(color);
    FillRect(dc, &rect, brush);
    DeleteObject(brush);
}
void Paint(HDC dc) {
    TRIVERTEX stops[2]{};
    stops[0].x = 0; stops[0].y = 0;
    stops[0].Red = 0x0c00; stops[0].Green = 0x1600; stops[0].Blue = 0x2c00;
    stops[1].x = W; stops[1].y = H;
    stops[1].Red = 0x0c00; stops[1].Green = 0x5900; stops[1].Blue = 0x6800;
    GRADIENT_RECT rect{0,1};
    GradientFill(dc, stops, 2, &rect, 1, GRADIENT_FILL_RECT_H);
    Solid(dc, RECT{0,175,W,178}, RGB(36,220,194));
    Print(dc, state.heading, RGB(250,252,255), L"RAGNAR", 40, 38, 640, 74);
    Print(dc, state.body, RGB(213,239,240),
          L"Ragnarok Online  /  Classic Visual Preview", 44, 119, 680, 30);
    Print(dc, state.smallFont, RGB(145,202,211),
          L"TURKUAZ   •   KURULUM   •   v0.1.0 QA", 44, 150, 680, 26);
    RECT content{};
    GetClientRect(state.hwnd, &content);
    const int footerY = std::max(460, content.bottom - 34);
    Solid(dc, RECT{40, footerY - 13, content.right - 40, footerY - 12}, RGB(39, 142, 157));
    Print(dc, state.smallFont, RGB(183,217,222),
          L"LevelUpGT  •  Yalnızca test  •  Oyun dosyaları değiştirilmez",
          42, footerY, content.right - 80, 26);
    switch (state.page) {
      case Page::Welcome:
        Print(dc, state.body, RGB(255,255,255),
              L"Ragnar dünyasına hoş geldin", 44, 218, 670, 38);
        Print(dc, state.smallFont, RGB(215,235,239),
              L"Bu kurulumda hedef klasörü ve kısayolları seçebilirsin.\n"
              L"Tüm adımlar bu pencerede tamamlanacak.", 44, 268, 670, 80, DT_WORDBREAK);
        Print(dc, state.smallFont, RGB(155,210,217),
              L"Önizleme sunucuya bağlanmaz veya oyun güncellemesi yapmaz.",
              44, 366, 680, 40);
        break;
      case Page::Options:
        Print(dc, state.body, RGB(255,255,255),
              L"Kurulum seçenekleri", 44, 203, 675, 37);
        Print(dc, state.smallFont, RGB(220,243,246),
              L"Kurulum klasörü", 44, 248, 670, 27);
        Print(dc, state.smallFont, RGB(220,243,246),
              L"Kısayollar", 44, 341, 670, 27);
        break;
      case Page::Installing: {
        Print(dc, state.body, RGB(255,255,255),
              L"Ragnar kuruluyor", 44, 214, 670, 40);
        Print(dc, state.smallFont, RGB(219,237,239),
              L"Dosyalar sessiz olarak kuruluyor. Bu pencereyi kapatmayın.",
              44, 270, 680, 54, DT_WORDBREAK);
        Solid(dc, RECT{44,354,732,368}, RGB(30,67,84));
        const int start = (state.pulse*9) % 780 - 90;
        const int left = std::max(44,44+start);
        const int right = std::min(732,44+start+160);
        if (right > left)
            Solid(dc, RECT{left,354,right,368}, RGB(44,225,192));
        Print(dc, state.smallFont, RGB(165,206,215),
              L"Yükleme sürüyor — gerçek yüzde bilgisi olmadığı için gösterilmez.",
              44, 386, 678, 48, DT_WORDBREAK);
        break;
      }
      case Page::Complete:
        Print(dc, state.body, RGB(255,255,255),
              L"Kurulum tamamlandı", 44, 220, 670, 45);
        Print(dc, state.smallFont, RGB(218,242,235),
              state.error.empty()
                  ? L"Ragnar Classic önizlemesi hazır. İstersen şimdi açabilir\n"
                    L"veya Kapat seçeneğiyle kurulumdan çıkabilirsin."
                  : state.error.c_str(),
              44, 278, 666, 72, DT_WORDBREAK);
        break;
      case Page::Failed:
        Print(dc, state.body, RGB(255,222,214),
              L"Kurulum tamamlanamadı", 44, 207, 680, 43);
        Print(dc, state.smallFont, RGB(247,217,218), state.error.c_str(),
              44, 268, 672, 148, DT_WORDBREAK | DT_END_ELLIPSIS);
        break;
    }
}
bool SaveTarget() {
    wchar_t buffer[1024]{};
    GetWindowTextW(state.folder, buffer, 1024);
    auto newTarget = std::wstring(buffer);
    if (!RagnarValidTarget(newTarget)) {
        MessageBoxW(state.hwnd,
                    L"Geçersiz klasör: yalnızca RagnarClassicTurkuazQA alt klasörüne kurulabilir.",
                    kTitle, MB_OK | MB_ICONWARNING);
        return false;
    }
    state.destination = newTarget;
    return true;
}
void DoWork(std::wstring target, bool desktop, bool start, HWND window) {
    // Shell link COM operations run on the background worker.
    const HRESULT comResult = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
    auto result = std::make_unique<Result>();
    auto temp = RagnarTempFolder();
    if (temp.empty()) result->message = L"Geçici klasör oluşturulamadı.";
    else {
        auto setup = temp + L"\\" + kSetupName;
        if (!RagnarExtractSetup(setup))
            result->message = L"Kurulum dosyası açılamadı veya SHA-256 uyuşmuyor.";
        else {
            DWORD exit = 0;
            if (!RagnarSilentInstall(setup, target, exit, result->message))
                result->message += L"\nLütfen çalışan Ragnar pencerelerini kapatıp tekrar dene.";
            else {
                result->success = true;
                bool shortcuts = true;
                if (desktop) shortcuts &= RagnarCreateShortcut(true,target);
                if (start) shortcuts &= RagnarCreateShortcut(false,target);
                if (!shortcuts) result->message =
                    L"Kurulum tamamlandı fakat bazı kısayollar oluşturulamadı.";
            }
        }
        DeleteFileW(setup.c_str());
        RemoveDirectoryW(temp.c_str());
    }
    if (SUCCEEDED(comResult)) CoUninitialize();
    PostMessageW(window, kDone, 0, reinterpret_cast<LPARAM>(result.release()));
}
void StartWork() {
    if (!SaveTarget()) return;
    if (!RagnarVerify()) {
        MessageBoxW(state.hwnd, L"Kurulum içeriği doğrulanamadı.",
                    kTitle, MB_OK | MB_ICONERROR);
        return;
    }
    const bool desktop = SendMessageW(state.desktop, BM_GETCHECK, 0, 0) == BST_CHECKED;
    const bool start = SendMessageW(state.start, BM_GETCHECK, 0, 0) == BST_CHECKED;
    SetPage(Page::Installing);
    std::thread(DoWork, state.destination, desktop, start, state.hwnd).detach();
}
void ShowFolderChooser() {
    IFileOpenDialog* dialog = nullptr;
    if (FAILED(CoCreateInstance(CLSID_FileOpenDialog, nullptr, CLSCTX_INPROC_SERVER,
                                IID_PPV_ARGS(&dialog)))) return;
    DWORD options = 0;
    dialog->GetOptions(&options);
    dialog->SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST);
    dialog->SetTitle(L"Ragnar kurulumunun ana klasörünü seç");
    if (SUCCEEDED(dialog->Show(state.hwnd))) {
        IShellItem* item = nullptr;
        if (SUCCEEDED(dialog->GetResult(&item)) && item) {
            PWSTR value = nullptr;
            if (SUCCEEDED(item->GetDisplayName(SIGDN_FILESYSPATH, &value)) && value) {
                std::wstring path = value;
                CoTaskMemFree(value);
                if (!path.empty() && path.back() == L'\\') path.pop_back();
                path += L"\\RagnarClassicTurkuazQA";
                SetWindowTextW(state.folder,path.c_str());
            }
            item->Release();
        }
    }
    dialog->Release();
}
LRESULT CALLBACK Proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
      case WM_CREATE: {
        state.hwnd = window;
        state.heading = CreateFontW(-55,0,0,0,FW_BOLD,FALSE,FALSE,FALSE,
                                    DEFAULT_CHARSET,0,0,CLEARTYPE_QUALITY,0,L"Segoe UI");
        state.body = CreateFontW(-24,0,0,0,FW_SEMIBOLD,FALSE,FALSE,FALSE,
                                 DEFAULT_CHARSET,0,0,CLEARTYPE_QUALITY,0,L"Segoe UI");
        state.smallFont = CreateFontW(-17,0,0,0,FW_NORMAL,FALSE,FALSE,FALSE,
                                  DEFAULT_CHARSET,0,0,CLEARTYPE_QUALITY,0,L"Segoe UI");
        state.formBackground = CreateSolidBrush(RGB(20,71,83));
        RECT client{};
        GetClientRect(window, &client);
        const int navY = client.bottom - 104;
        state.go = CreateWindowW(L"BUTTON",L"DEVAM ET  →",
                                 WS_CHILD|WS_VISIBLE|BS_PUSHBUTTON,
                                 515,navY,220,42,window,reinterpret_cast<HMENU>(kGo),nullptr,nullptr);
        state.back = CreateWindowW(L"BUTTON",L"GERİ",WS_CHILD|BS_PUSHBUTTON,
                                   372,navY,125,42,window,reinterpret_cast<HMENU>(kBack),nullptr,nullptr);
        state.folder = CreateWindowExW(WS_EX_CLIENTEDGE,L"EDIT",L"",
                                      WS_CHILD|ES_AUTOHSCROLL|WS_TABSTOP,
                                      44,278,563,32,window,reinterpret_cast<HMENU>(kPath),nullptr,nullptr);
        state.browse = CreateWindowW(L"BUTTON",L"Gözat...",WS_CHILD|BS_PUSHBUTTON,
                                     617,278,118,32,window,reinterpret_cast<HMENU>(kBrowse),nullptr,nullptr);
        state.desktop = CreateWindowW(L"BUTTON",L"Masaüstü kısayolu",
                                      WS_CHILD|BS_AUTOCHECKBOX|WS_TABSTOP,
                                      44,378,290,29,window,reinterpret_cast<HMENU>(kDesktop),nullptr,nullptr);
        state.start = CreateWindowW(L"BUTTON",L"Başlat menüsü kısayolu",
                                    WS_CHILD|BS_AUTOCHECKBOX|WS_TABSTOP,
                                    355,378,320,29,window,reinterpret_cast<HMENU>(kStart),nullptr,nullptr);
        for (HWND child : {state.go,state.back,state.folder,state.browse,state.desktop,state.start})
            SendMessageW(child,WM_SETFONT,reinterpret_cast<WPARAM>(state.smallFont),TRUE);
        state.destination = RagnarDefaultTarget();
        SetWindowTextW(state.folder,state.destination.c_str());
        SendMessageW(state.desktop,BM_SETCHECK,BST_CHECKED,0);
        SendMessageW(state.start,BM_SETCHECK,BST_CHECKED,0);
        SetPage(Page::Welcome);
        return 0;
      }
      case WM_ERASEBKGND: return TRUE;
      case WM_PAINT: {
        PAINTSTRUCT ps{};
        HDC dc = BeginPaint(window,&ps);
        Paint(dc);
        EndPaint(window,&ps);
        return 0;
      }
      case WM_CTLCOLOREDIT: {
        HDC dc = reinterpret_cast<HDC>(wparam);
        SetTextColor(dc,RGB(9,43,50)); SetBkColor(dc,RGB(229,244,245));
        return reinterpret_cast<LRESULT>(GetSysColorBrush(COLOR_WINDOW));
      }
      case WM_CTLCOLORBTN: {
        HDC dc = reinterpret_cast<HDC>(wparam);
        SetTextColor(dc, RGB(244,247,248)); SetBkMode(dc,TRANSPARENT);
        return reinterpret_cast<LRESULT>(state.formBackground);
      }
      case WM_TIMER:
        if (wparam == kPulse && state.page == Page::Installing) { state.pulse++; Redraw(); }
        return 0;
      case kDone: {
        std::unique_ptr<Result> completed(reinterpret_cast<Result*>(lparam));
        state.error = completed->message;
        SetPage(completed->success ? Page::Complete : Page::Failed);
        return 0;
      }
      case WM_COMMAND:
        switch (LOWORD(wparam)) {
          case kGo:
            if (state.page == Page::Welcome) SetPage(Page::Options);
            else if (state.page == Page::Options) StartWork();
            else if (state.page == Page::Failed) SetPage(Page::Options);
            else if (state.page == Page::Complete) {
                DWORD error = 0;
                if (RagnarOpenInstalled(state.destination,error)) DestroyWindow(window);
                else MessageBoxW(window,
                    (L"Uygulama başlatılamadı. Hata kodu: "+std::to_wstring(error)).c_str(),
                    kTitle, MB_ICONERROR | MB_OK);
            }
            break;
          case kBack:
            if (state.page == Page::Options) SetPage(Page::Welcome);
            else if (state.page == Page::Complete || state.page == Page::Failed)
                DestroyWindow(window);
            break;
          case kBrowse:
            if (state.page == Page::Options) ShowFolderChooser();
            break;
        }
        return 0;
      case WM_CLOSE:
        if (state.page == Page::Installing) {
            MessageBoxW(window,L"Kurulum devam ediyor, lütfen tamamlanmasını bekle.",
                        kTitle,MB_OK|MB_ICONINFORMATION);
            return 0;
        }
        DestroyWindow(window);
        return 0;
      case WM_DESTROY:
        KillTimer(window,kPulse);
        for (HFONT font : {state.heading,state.body,state.smallFont}) if (font) DeleteObject(font);
        if (state.formBackground) DeleteObject(state.formBackground);
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProcW(window,message,wparam,lparam);
}
bool SetupWindow(HINSTANCE instance, bool visible) {
    WNDCLASSEXW cls{sizeof(cls)};
    cls.hInstance=instance; cls.lpfnWndProc=Proc;
    cls.lpszClassName=L"RagnarTurkuazPreviewSetup";
    cls.hCursor=LoadCursorW(nullptr,IDC_ARROW);
    cls.hIcon=LoadIconW(instance,MAKEINTRESOURCEW(1));
    if (!RegisterClassExW(&cls) && GetLastError()!=ERROR_CLASS_ALREADY_EXISTS) return false;
    RECT work{};
    if (!SystemParametersInfoW(SPI_GETWORKAREA,0,&work,0)) {
        work = RECT{0,0,GetSystemMetrics(SM_CXSCREEN),GetSystemMetrics(SM_CYSCREEN)};
    }
    // Keep the full window inside the available work area, including taskbar.
    const int width = std::min(W,work.right-work.left);
    const int height = std::min(H,work.bottom-work.top);
    const int x=work.left+(work.right-work.left-width)/2;
    const int y=work.top+(work.bottom-work.top-height)/2;
    HWND window=CreateWindowExW(WS_EX_APPWINDOW,cls.lpszClassName,kTitle,
                               WS_CAPTION|WS_SYSMENU|WS_MINIMIZEBOX,
                               x,y,width,height,nullptr,nullptr,instance,nullptr);
    if (!window) return false;
    if (visible) { ShowWindow(window,SW_SHOW); UpdateWindow(window); }
    return true;
}
} // namespace

int WINAPI wWinMain(HINSTANCE instance,HINSTANCE,LPWSTR arguments,int) {
    if (arguments && wcscmp(arguments,L"--verify-payload")==0)
        return RagnarVerify()?0:4;
    // Hidden real-window construction smoke: validates options/buttons without installing.
    if (arguments && wcscmp(arguments,L"--verify-ui-contract")==0) {
        HRESULT init=CoInitializeEx(nullptr,COINIT_APARTMENTTHREADED);
        if (FAILED(init)) return 4;
        const bool success=SetupWindow(instance,false)&&state.hwnd&&state.folder
            &&state.desktop&&state.start&&state.go;
        if (success) DestroyWindow(state.hwnd);
        CoUninitialize();
        return success?0:4;
    }
    if (arguments && wcscmp(arguments,L"--verify-dialog-runtime")==0)
        return RagnarDialogAvailable()?0:4;
    if (arguments && *arguments) return 2;
    HRESULT init=CoInitializeEx(nullptr,COINIT_APARTMENTTHREADED);
    if (FAILED(init)) return 4;
    if (!RagnarVerify()) {
        MessageBoxW(nullptr,L"Kurulum paketi bozuk: SHA-256 doğrulanamadı.",
                    kTitle,MB_OK|MB_ICONERROR);
        CoUninitialize();return 4;
    }
    if (!SetupWindow(instance,true)) { CoUninitialize();return 4; }
    MSG msg{};
    while (GetMessageW(&msg,nullptr,0,0)>0) {
        TranslateMessage(&msg); DispatchMessageW(&msg);
    }
    CoUninitialize();
    return static_cast<int>(msg.wParam);
}
