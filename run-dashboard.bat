@echo off
setlocal
pushd "%~dp0" || exit /b 1

where cargo >nul 2>&1
if errorlevel 1 (
    echo Cargo was not found. Install the Rust toolchain specified in rust-toolchain.toml.
    goto :failed
)
where cmake >nul 2>&1
if errorlevel 1 (
    echo CMake was not found. Install CMake 3.24 or newer.
    goto :failed
)

rem Prefer a matching installed stable toolchain when rustup has no version alias.
set "CARGO_TOOLCHAIN="
for /f "tokens=2" %%V in ('rustc +stable --version 2^>nul') do if "%%V"=="1.99.1" set "CARGO_TOOLCHAIN=+stable"
set "CARGO_TARGET_DIR=%CD%\target"

rem Respect an explicitly configured Qt kit, or find a standard Qt MSVC kit.
if not defined Qt6_DIR if not defined CMAKE_PREFIX_PATH (
    for /d %%D in ("C:\Qt\6.*") do (
        if exist "%%~fD\msvc2022_64\lib\cmake\Qt6\Qt6Config.cmake" set "CMAKE_PREFIX_PATH=%%~fD\msvc2022_64"
    )
)
if not defined Qt6_DIR if not defined CMAKE_PREFIX_PATH (
    echo Qt 6.5 or newer for MSVC was not found. Set Qt6_DIR or CMAKE_PREFIX_PATH.
    goto :failed
)

echo Building the Rust runtime...
call cargo %CARGO_TOOLCHAIN% build --release --locked --bin cgacp-runtime
if errorlevel 1 goto :failed

echo Building the desktop dashboard...
cmake -S . -B target\dashboard-windows\build -G "Visual Studio 17 2022" -A x64 "-DCGACP_RUNTIME_EXECUTABLE=%CARGO_TARGET_DIR%\release\cgacp-runtime.exe" -DBUILD_TESTING=OFF -DCGACP_DEPLOY_QT=ON
if errorlevel 1 goto :failed
cmake --build target\dashboard-windows\build --config Release --target cgacp-stage-runtime --parallel
if errorlevel 1 goto :failed
cmake --install target\dashboard-windows\build --config Release --prefix "%CARGO_TARGET_DIR%\dashboard-windows\stage"
if errorlevel 1 goto :failed

"%CARGO_TARGET_DIR%\dashboard-windows\stage\bin\cgacp-dashboard.exe" %*
set "result=%errorlevel%"
popd
exit /b %result%

:failed
echo Dashboard launch failed. Review the error above.
popd
exit /b 1
