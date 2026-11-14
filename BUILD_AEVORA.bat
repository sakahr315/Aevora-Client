@echo off
setlocal
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
    echo Rust and Cargo are required. Install them from https://rustup.rs
    exit /b 1
)

echo Building Aevora Client in release mode...
cargo build --release --bin aevora-client
if errorlevel 1 (
    echo.
    echo Aevora Client build failed. Scroll up for the Cargo error.
    exit /b 1
)

echo Building the Luau runner...
cargo build --manifest-path tools\aevora-luau\Cargo.toml --release
if errorlevel 1 (
    echo.
    echo Luau runner build failed. Scroll up for the Cargo error.
    exit /b 1
)

echo.
echo Build complete:
echo %~dp0target\release\aevora-client.exe
echo %~dp0tools\aevora-luau\target\release\aevora-luau.exe
exit /b 0
