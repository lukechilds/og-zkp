#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
TARGETS=(x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu aarch64-apple-darwin)
IMAGE="${IMAGE:-ghcr.io/lukechilds/og-zkp:latest}"
DIST="$ROOT/dist"

# Host detection
HOST_OS="$(uname -s)"
HOST_ARCH="$(uname -m)"
case "$HOST_ARCH" in
    arm64) HOST_ARCH=aarch64 ;;
esac

target_os() {
    case "$1" in
        *linux*)  echo linux ;;
        *darwin*) echo darwin ;;
    esac
}

target_arch() {
    case "$1" in
        x86_64*)  echo x86_64 ;;
        aarch64*) echo aarch64 ;;
    esac
}

docker_arch() {
    case "$1" in
        x86_64*)  echo amd64 ;;
        aarch64*) echo arm64 ;;
    esac
}

is_native() {
    local t_os t_arch
    t_os="$(target_os "$1")"
    t_arch="$(target_arch "$1")"
    [[ "$HOST_OS" == "Linux" && "$t_os" == "linux" && "$HOST_ARCH" == "$t_arch" ]] ||
    [[ "$HOST_OS" == "Darwin" && "$t_os" == "darwin" && "$HOST_ARCH" == "$t_arch" ]]
}

cmd_guest() {
    RISC0_USE_DOCKER=1 cargo build -p methods --release
}

cmd_build_one() {
    local target="$1"
    if is_native "$target"; then
        echo "Building $target (native)"
        RISC0_SKIP_BUILD=1 CARGO_TARGET_DIR="$ROOT/target" \
            cargo build --release --target "$target"
    else
        local arch
        arch="$(docker_arch "$target")"
        echo "Building $target (docker linux/$arch)"
        docker run --rm \
            --platform "linux/$arch" \
            -v "$ROOT":/app -w /app \
            -v cargo-cache:/usr/local/cargo/registry \
            -e RISC0_SKIP_BUILD=1 \
            -e CARGO_TARGET_DIR=/app/target \
            rust:bookworm \
            cargo build --release --target "$target"
    fi
    mkdir -p "$DIST"
    cp "$ROOT/target/$target/release/og-zkp" "$DIST/og-zkp-$target"
}

cmd_build() {
    if [[ $# -gt 0 ]]; then
        cmd_build_one "$1"
    else
        for t in "${TARGETS[@]}"; do
            cmd_build_one "$t"
        done
    fi
}

cmd_docker() {
    cp "$DIST/og-zkp-x86_64-unknown-linux-gnu" "$ROOT/docker/og-zkp"
    docker build --platform linux/amd64 -t "$IMAGE" "$ROOT/docker/"
    rm "$ROOT/docker/og-zkp"
}

cmd_checksums() {
    cd "$DIST"
    if command -v sha256sum &>/dev/null; then
        sha256sum og-zkp-* | tee sha256sums.txt
    else
        shasum -a 256 og-zkp-* | tee sha256sums.txt
    fi
}

cmd_release() {
    cmd_guest
    cmd_build
    cmd_docker
    cmd_checksums
}

cmd_clean() {
    rm -rf "$DIST"
    cargo clean
}

case "${1:-}" in
    guest)     cmd_guest ;;
    host)      shift; cmd_build "$@" ;;
    docker)    cmd_docker ;;
    checksums) cmd_checksums ;;
    release)   cmd_release ;;
    clean)     cmd_clean ;;
    *)
        echo "Usage: $0 {guest|host [target]|docker|checksums|release|clean}"
        exit 1
        ;;
esac
