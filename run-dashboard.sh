#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$root"

if ! command -v cargo >/dev/null 2>&1; then
    echo "Cargo was not found. Install the Rust toolchain specified in rust-toolchain.toml." >&2
    exit 1
fi
if ! command -v cmake >/dev/null 2>&1; then
    echo "CMake was not found. Install CMake 3.24 or newer." >&2
    exit 1
fi

export CARGO_TARGET_DIR="$root/target"
if rustc +stable --version 2>/dev/null | grep -q '^rustc 1\.99\.1 '; then
    cargo +stable build --release --locked --bin cgacp-runtime
else
    cargo build --release --locked --bin cgacp-runtime
fi

build_dir="$CARGO_TARGET_DIR/dashboard-linux/build"
cmake -S "$root" -B "$build_dir" \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_TESTING=OFF \
    "-DCGACP_RUNTIME_EXECUTABLE=$CARGO_TARGET_DIR/release/cgacp-runtime"
cmake --build "$build_dir" --config Release --target cgacp-stage-runtime --parallel

dashboard="$build_dir/desktop/cgacp-dashboard"
if [ ! -f "$dashboard" ] && [ -f "$build_dir/desktop/Release/cgacp-dashboard" ]; then
    dashboard="$build_dir/desktop/Release/cgacp-dashboard"
fi
exec "$dashboard" "$@"
