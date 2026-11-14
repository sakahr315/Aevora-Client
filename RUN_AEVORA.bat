@echo off
setlocal
if not exist "%~dp0target\release\aevora-client.exe" (
    echo Aevora has not been built yet.
    echo Run BUILD_AEVORA.bat first.
    exit /b 1
)
start "" "%~dp0target\release\aevora-client.exe"
