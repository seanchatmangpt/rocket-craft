@echo off
setlocal enabledelayedexpansion

:: rocket - Unified Rocket Craft Management Tool (Rust Wrapper for Windows)

set "PROJECT_ROOT=%~dp0"
set "BIN_PATH=%PROJECT_ROOT%tools\target\release\rocket-cmd.exe"

:: Build the tool if it doesn't exist
if not exist "%BIN_PATH%" (
    echo Building Rocket Craft tools (release)...
    pushd "%PROJECT_ROOT%tools\rocket-cmd"
    cargo build --release
    popd
)

:: Pass all arguments to the Rust binary
"%BIN_PATH%" %*
