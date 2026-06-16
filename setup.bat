@echo off
setlocal enabledelayedexpansion

:: Rocket Craft Setup Proxy Script for Windows
:: Ensures Rust is installed and proxies to 'rocket setup'

echo ================================================
echo       Rocket Craft Project Bootstrapper         
echo ================================================

:: 1. Check for Rust/Cargo
cargo --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Rust/Cargo not found. Attempting to install...
    
    :: Use PowerShell to download rustup-init
    echo Downloading rustup-init.exe...
    powershell -Command "Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe"
    
    if exist rustup-init.exe (
        echo Running rustup-init.exe...
        :: Run rustup-init with -y for non-interactive installation
        rustup-init.exe -y
        del rustup-init.exe
        
        :: Refresh PATH for the current session
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
        
        cargo --version >nul 2>&1
        if %ERRORLEVEL% equ 0 (
            echo Rust installed successfully!
        ) else (
            echo Rust installation failed or requires a new terminal.
            echo Please install it manually from https://rustup.rs/
            pause
            exit /b 1
        )
    else (
        echo Error: Failed to download rustup-init.exe
        echo Please install Rust manually from https://rustup.rs/
        pause
        exit /b 1
    )
) else (
    echo [OK] Rust/Cargo detected.
)

:: 2. Proxy to rocket setup
if exist "%~dp0rocket.bat" (
    echo Proxying to rocket setup...
    call "%~dp0rocket.bat" setup
) else (
    echo Error: 'rocket.bat' not found in %~dp0
    pause
    exit /b 1
)

exit /b 0
