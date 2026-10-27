; Dosya Yolu: installer/windows/BeamPatcher-QA.iss
; Amac: Eski offline Inno QA taslagi; dagitim icin kullanilmaz
; Modul - Inno Setup / Windows test artifact
; Version: 0.1.0-candidate
; Aciklama: Patcher.exe ve QA config paketi, oyun istemcisi/GRF/secret yok
; Bagimli Oldugu Katman: installer/windows/config.qa.yml | beam-patcher/src/main.rs

#ifndef BeamExe
#define BeamExe "..\..\target\release\beam-patcher.exe"
#endif
#ifndef SetupOut
#define SetupOut "..\..\build\windows-setup-qa"
#endif
#define AppName "Turkuaz Ragnarok Online QA"
#define AppVersion "1.0.1"
#define ExeName "beam-patcher.exe"

[Setup]
AppId={{ED6D13DA-2C9C-4BF7-9C36-838A416B1B45}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=TurkuazSoft
DefaultDirName={localappdata}\Programs\TurkuazSoft\Turkuaz Ragnarok Online QA
DefaultGroupName=Turkuaz Ragnarok Online QA
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#SetupOut}
OutputBaseFilename=TurkuazRagnarokOnline-Inno-DEPRECATED-QA-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName={#AppName} (TEST ONLY)
CloseApplications=yes
RestartApplications=no
ChangesEnvironment=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "tr"; MessagesFile: "compiler:Languages\Turkish.isl"

[Files]
Source: "{#BeamExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "config.qa.yml"; DestDir: "{app}"; DestName: "config.yml"; Flags: onlyifdoesntexist uninsneveruninstall
Source: "README-QA.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Turkuaz Ragnarok Online QA"; Filename: "{app}\{#ExeName}"; WorkingDir: "{app}"
Name: "{group}\Read QA Safety Guide"; Filename: "{app}\README-QA.txt"
Name: "{group}\Uninstall Beam QA"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Turkuaz Ragnarok Online QA"; Filename: "{app}\{#ExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create QA desktop shortcut"; GroupDescription: "Additional tasks:"

[Run]
Filename: "{app}\README-QA.txt"; Description: "Read test-only safety instructions"; Flags: shellexec postinstall unchecked skipifsilent
