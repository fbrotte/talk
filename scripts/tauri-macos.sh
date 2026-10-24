#!/usr/bin/env bash
set -euo pipefail

# Builds on a Mac. Whisper runs on the GPU through Metal there, so the Vulkan feature the
# other platforms build with by default is left out:
#
#     scripts/tauri-macos.sh dev | build | check | test

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -x "${HOME:-}/.cargo/bin/cargo" ]]; then
  export PATH="${HOME}/.cargo/bin:$PATH"
fi

# src-tauri/.cargo/config.toml names the MSVC compilers for Windows. Cargo leaves a
# variable the environment already sets alone, so Apple's own win on a Mac.
export CC="${CC:-cc}" CXX="${CXX:-c++}" CMAKE_GENERATOR="${CMAKE_GENERATOR:-Unix Makefiles}"

cd "$repo_dir"
case "${1:-}" in
  dev) shift; exec bunx tauri dev --features metal "$@" -- --no-default-features ;;
  build) shift; exec bunx tauri build --features metal "$@" -- --no-default-features ;;
  check) shift; cd src-tauri; exec cargo check --no-default-features --features metal "$@" ;;
  test) shift; cd src-tauri; exec cargo test --no-default-features --features metal "$@" ;;
  *) echo "usage: $0 dev|build|check|test [args]" >&2; exit 2 ;;
esac
