@echo off
REM Dosya Yolu: tools/windows/ragnar-turkuaz-qa/Uninstall-RagnarTurkuazQA.cmd
REM Amac: Remove isolated preview through the real TurkuazInstaller CLI
REM Modul - Windows CMD
REM Version: 0.1.0-qa
REM Bagimli Oldugu Katman: Tool | TurkuazInstaller CLI
setlocal
set "QA_ROOT=%~dp0"
"%QA_ROOT%cli\TurkuazInstaller.Cli.exe" uninstall --package levelupgt.ragnar-classic-preview
if errorlevel 1 (
  echo Uninstall failed; keeping QA config and trust as-is for troubleshooting.
  exit /b 4
)
"%QA_ROOT%qa\Ragnar.QaProvisioner.exe" cleanup "%QA_ROOT%"
if errorlevel 1 exit /b 4
echo Ragnar QA preview uninstalled and owned QA-only config cleaned up.
exit /b 0
