; Dosya Yolu: REPO_ROOT/installer/windows/BeamPatcher-NSIS-Backup.nsi
; Amac: TurkuazInstaller yaninda sadece private kurtarma icin NSIS yedek paketi
; Modul - NSIS fallback installer
; Version: 1.0.1-qa
; Aciklama: Burak-INS-001; ayni kaynak/artifact, QA profile, no default public distribution
; Bagimli Oldugu Katman: .github/workflows/dual-installer-policy.yml | config.qa.yml
Unicode true
RequestExecutionLevel user
SetCompressor /SOLID lzma

!ifndef BeamExe
  !error "BeamExe must point to the pinned QA beam-patcher.exe"
!endif
!ifndef NsisOut
  !error "NsisOut must point to an explicitly created private output directory"
!endif

Name "LevelUp Games Beam QA - NSIS BACKUP ONLY"
OutFile "${NsisOut}\LevelUpGames-Beam-NSIS-BACKUP-QA-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\LevelUpGT\Beam Patcher QA-NSIS-Backup"
ShowInstDetails show
ShowUninstDetails show

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install NSIS backup QA" SecInstall
  SetShellVarContext current
  SetOutPath "$INSTDIR"
  SetOverwrite on
  File "/oname=beam-patcher.exe" "${BeamExe}"
  File "/oname=README-QA.txt" "installer\windows\README-QA.txt"
  SetOverwrite off
  File "/oname=config.yml" "installer\windows\config.qa.yml"
  SetOverwrite on

  WriteUninstaller "$INSTDIR\Uninstall-NSIS-Backup.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LevelUpGamesBeamQaNsisBackup" "DisplayName" "LevelUp Games Beam QA (NSIS backup)"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LevelUpGamesBeamQaNsisBackup" "UninstallString" '"$INSTDIR\Uninstall-NSIS-Backup.exe"'
  CreateDirectory "$SMPROGRAMS\LevelUp Games QA NSIS Backup"
  CreateShortcut "$SMPROGRAMS\LevelUp Games QA NSIS Backup\Beam QA (Backup).lnk" "$INSTDIR\beam-patcher.exe"
  CreateShortcut "$SMPROGRAMS\LevelUp Games QA NSIS Backup\Uninstall Backup.lnk" "$INSTDIR\Uninstall-NSIS-Backup.exe"
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  Delete "$SMPROGRAMS\LevelUp Games QA NSIS Backup\Beam QA (Backup).lnk"
  Delete "$SMPROGRAMS\LevelUp Games QA NSIS Backup\Uninstall Backup.lnk"
  RMDir "$SMPROGRAMS\LevelUp Games QA NSIS Backup"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\LevelUpGamesBeamQaNsisBackup"
  Delete "$INSTDIR\beam-patcher.exe"
  Delete "$INSTDIR\README-QA.txt"
  Delete "$INSTDIR\Uninstall-NSIS-Backup.exe"
  ; User config, external QA GRF and client directories are never erased.
  RMDir "$INSTDIR"
SectionEnd
