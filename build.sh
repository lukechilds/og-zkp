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

docker_target() {
    case "$1" in
        amd64|linux/amd64) echo x86_64-unknown-linux-gnu ;;
        arm64|linux/arm64) echo aarch64-unknown-linux-gnu ;;
        *)                 echo "$1" ;;
    esac
}

native_linux_target() {
    case "$HOST_ARCH" in
        x86_64)  echo x86_64-unknown-linux-gnu ;;
        aarch64) echo aarch64-unknown-linux-gnu ;;
        *)       echo "Unsupported host architecture for Docker: $HOST_ARCH" >&2; exit 1 ;;
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
    cargo build -p methods --release
}

cmd_build_one() {
    local target="$1"
    local package="${2:-og-zkp}"
    local binary_name
    case "$package" in
        og-zkp-verifier) binary_name="og-zkp-verifier" ;;
        *)               binary_name="og-zkp" ;;
    esac
    if is_native "$target"; then
        echo "Building $package for $target (native)"
        RISC0_SKIP_BUILD=1 CARGO_TARGET_DIR="$ROOT/target" \
            cargo build --release --target "$target" -p "$package"
    else
        local arch
        arch="$(docker_arch "$target")"
        echo "Building $package for $target (docker linux/$arch)"
        docker run --rm \
            --platform "linux/$arch" \
            -v "$ROOT":/app -w /app \
            -v cargo-cache:/usr/local/cargo/registry \
            -e RISC0_SKIP_BUILD=1 \
            -e CARGO_TARGET_DIR=/app/target \
            rust:1.88.0-trixie \
            cargo build --release --target "$target" -p "$package"
    fi
    mkdir -p "$DIST"
    cp "$ROOT/target/$target/release/$binary_name" "$DIST/$binary_name-$target"
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

cmd_verifier() {
    local target="${1:-x86_64-unknown-linux-gnu}"
    cmd_build_one "$target" og-zkp-verifier
}

cmd_docker() {
    local target="${1:-native}"
    if [[ "$target" == "native" ]]; then
        target="$(native_linux_target)"
    else
        target="$(docker_target "$target")"
    fi

    local arch platform src staged
    arch="$(docker_arch "$target")"
    platform="linux/$arch"
    src="$DIST/og-zkp-$target"
    staged="$ROOT/docker/og-zkp-$arch"

    if [[ ! -f "$src" ]]; then
        echo "Error: missing $src" >&2
        echo "Run: ./build.sh host $target" >&2
        exit 1
    fi

    cp "$src" "$staged"
    trap 'rm -f "$staged"; trap - RETURN' RETURN
    docker build --platform "$platform" -t "$IMAGE" "$ROOT/docker/"
}

cmd_docker_verifier() {
    local image="${VERIFIER_IMAGE:-ghcr.io/lukechilds/og-zkp-verifier:latest}"
    cp "$DIST/og-zkp-verifier-x86_64-unknown-linux-gnu" "$ROOT/docker/og-zkp-verifier"
    docker build --platform linux/amd64 -t "$image" -f "$ROOT/docker/Dockerfile.verifier" "$ROOT/docker/"
    rm "$ROOT/docker/og-zkp-verifier"
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

cmd_blockhashes() {
    local rpc_url="${RPC_URL:-http://127.0.0.1:8332}"
    local output="$ROOT/core/src/blockhashes.txt"
    local batch_size=500
    local max_retries=5

    # Build auth: user/pass takes priority, then cookie file
    local auth_header=""
    if [[ -n "${RPC_USER:-}" && -n "${RPC_PASS:-}" ]]; then
        auth_header="Authorization: Basic $(printf '%s:%s' "$RPC_USER" "$RPC_PASS" | base64)"
    else
        local cookie_file="${RPC_COOKIE:-$HOME/.bitcoin/.cookie}"
        if [[ ! -f "$cookie_file" ]]; then
            echo "Error: cookie file not found at $cookie_file" >&2
            echo "Set RPC_USER/RPC_PASS or RPC_COOKIE" >&2
            exit 1
        fi
        auth_header="Authorization: Basic $(base64 < "$cookie_file")"
    fi

    rpc_call() {
        local data="$1"
        local attempt
        for (( attempt=1; attempt<=max_retries; attempt++ )); do
            local result
            if result=$(curl -sf --max-time 120 -X POST \
                -H "Content-Type: application/json" \
                -H "$auth_header" \
                -d "$data" \
                "$rpc_url"); then
                echo "$result"
                return 0
            fi
            echo "Request failed (attempt $attempt/$max_retries), retrying in ${attempt}s..." >&2
            sleep "$attempt"
        done
        echo "Request failed after $max_retries attempts" >&2
        return 1
    }

    # Get chain height
    local height
    height=$(rpc_call '{"jsonrpc":"1.0","method":"getblockcount","params":[]}' | jq -r '.result')
    echo "Fetching $((height + 1)) block hashes in batches of $batch_size..." >&2

    > "$output"
    for (( start=0; start<=height; start+=batch_size )); do
        local end=$(( start + batch_size - 1 ))
        if (( end > height )); then end=$height; fi

        # Build batch JSON-RPC request
        local batch="["
        for (( i=start; i<=end; i++ )); do
            if (( i > start )); then batch+=","; fi
            batch+="{\"jsonrpc\":\"1.0\",\"id\":$i,\"method\":\"getblockhash\",\"params\":[$i]}"
        done
        batch+="]"

        rpc_call "$batch" | jq -r 'sort_by(.id) | .[].result' >> "$output"

        if (( end > 0 && (end + 1) % 10000 < batch_size )); then
            printf '%d/%d blocks (%.1f%%)\n' "$((end + 1))" "$((height + 1))" \
                "$(echo "($end + 1) * 100 / ($height + 1)" | bc -l)" >&2
        fi
    done

    echo "Wrote $((height + 1)) block hashes to $output" >&2
}

cmd_clean() {
    rm -rf "$DIST"
    cargo clean
}

case "${1:-}" in
    guest)            cmd_guest ;;
    host)             shift; cmd_build "$@" ;;
    verifier)         shift; cmd_verifier "$@" ;;
    docker)           cmd_docker ;;
    docker-verifier)  cmd_docker_verifier ;;
    checksums)        cmd_checksums ;;
    blockhashes)      shift; cmd_blockhashes "$@" ;;
    release)          cmd_release ;;
    clean)            cmd_clean ;;
    *)
        echo "Usage: $0 {guest|host [target]|verifier [target]|docker [target|native]|docker-verifier|checksums|blockhashes|release|clean}"
        exit 1
        ;;
esac
