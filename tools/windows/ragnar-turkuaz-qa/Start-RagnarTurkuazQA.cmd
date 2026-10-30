@echo off
REM Dosya Yolu: tools/windows/ragnar-turkuaz-qa/Start-RagnarTurkuazQA.cmd
REM Amac: Unprivileged local-only TurkuazInstaller + Ragnar preview setup
REM Modul - Windows CMD
REM Version: 0.1.0-qa
REM Bagimli Oldugu Katman: Tool | TurkuazInstaller CLI
setlocal
set "QA_ROOT=%~dp0"
if not exist "%QA_ROOT%qa\Ragnar.QaProvisioner.exe" (
  echo QA provisioner missing. Extract the full Windows artifact ZIP first.
  exit /b 2
)
if not exist "%QA_ROOT%cli\TurkuazInstaller.Cli.exe" (
  echo TurkuazInstaller CLI missing. Do not run from inside the ZIP.
  exit /b 2
)
"%QA_ROOT%qa\Ragnar.QaProvisioner.exe" prepare "%QA_ROOT%"
if errorlevel 1 (
  echo QA preparation failed. Existing TurkuazInstaller settings are never overwritten.
  exit /b 4
)
"%QA_ROOT%cli\TurkuazInstaller.Cli.exe" install --package levelupgt.ragnar-classic-preview --manifest "%QA_ROOT%qa\installer-manifest.yml" --channel stable --target "%LOCALAPPDATA%\Programs\RagnarClassicTurkuazQA"
if errorlevel 1 (
  echo Install did not complete. Read CLI diagnostics above.
  exit /b 4
)
echo Installation completed through TurkuazInstaller CLI.
echo Opening the separately branded TurkuazInstaller management UI.
start "" "%QA_ROOT%TurkuazInstaller.Bootstrapper.exe"
exit /b 0
