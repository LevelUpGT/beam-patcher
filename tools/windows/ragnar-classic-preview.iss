; Dosya Yolu: tools/windows/ragnar-classic-preview.iss
; Amac: Install a disabled-backend Beam Ragnar Classic UI demo independently of the game
; Modul - Inno Setup 6 Windows install
; Version: 0.1.0-qa
; Aciklama: Installs UI test EXE only, never target game folder or GRF files.
; Bagimli Oldugu Katman: cargo build --features preview-only | beam-ui/public

#define Application "Ragnar Classic Beam UI Preview"
#define AppVersion "0.1.0-preview"

[Setup]
AppId={{D826A86C-534C-4B89-94E9-26A6A2209B31}
AppName={#Application}
AppVersion={#AppVersion}
AppPublisher=LevelUpGT QA
AppPublisherURL=https://github.com/LevelUpGT/beam-patcher
DefaultDirName={localappdata}\Programs\RagnarClassicBeamPreview
DefaultGroupName={#Application}
PrivilegesRequired=lowest
DisableProgramGroupPage=yes
Compression=lzma2
SolidCompression=yes
OutputDir=..\..\dist\ragnar-classic-preview
OutputBaseFilename=RagnarClassicBeamPreviewSetup-0.1.0
UninstallDisplayIcon={app}\RagnarClassicPreview.exe
WizardStyle=modern
UsePreviousAppDir=no
CloseApplications=yes

[Files]
Source: "..\..\target\release\beam-patcher.exe"; DestDir: "{app}"; DestName: "RagnarClassicPreview.exe"; Flags: ignoreversion

[Icons]
Name: "{group}\{#Application}"; Filename: "{app}\RagnarClassicPreview.exe"
Name: "{autodesktop}\{#Application}"; Filename: "{app}\RagnarClassicPreview.exe"; Tasks: desktopicon

[Tasks]
Name: desktopicon; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Run]
Filename: "{app}\RagnarClassicPreview.exe"; Description: "Open Ragnar Classic preview"; Flags: nowait postinstall skipifsilent

[Code]
function InitializeSetup: Boolean;
begin
  Result := True;
end;
