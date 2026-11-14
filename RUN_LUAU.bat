@echo off
setlocal
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
    echo Rust and Cargo are required. Install them from https://rustup.rs
    exit /b 1
)

if "%~1"=="" (
    echo Usage:
    echo   RUN_LUAU.bat --list
    echo   RUN_LUAU.bat --script scripts\examples\hello.luau
    echo   RUN_LUAU.bat --script scripts\examples\worker-info.luau --instances 8
    exit /b 2
)

if exist "%~dp0tools\aevora-luau\target\release\aevora-luau.exe" goto run_built
cargo run --manifest-path tools\aevora-luau\Cargo.toml -- %*
exit /b %errorlevel%

:run_built
"%~dp0tools\aevora-luau\target\release\aevora-luau.exe" %*
exit /b %errorlevel%
