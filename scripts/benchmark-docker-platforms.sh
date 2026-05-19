#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-dev-prove}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RESULTS_DIR="${RESULTS_DIR:-$ROOT/bench-results/$(date +%Y%m%d-%H%M%S)}"

NATIVE_IMAGE="${NATIVE_IMAGE:-ghcr.io/lukechilds/og-zkp:apple-silicon-local}"
AMD64_IMAGE="${AMD64_IMAGE:-ghcr.io/lukechilds/og-zkp:amd64-baseline}"
PUBLIC_IMAGE="${PUBLIC_IMAGE:-ghcr.io/lukechilds/og-zkp:latest}"

STAT_INTERVAL="${STAT_INTERVAL:-2}"
TIMEOUT_SECONDS="${TIMEOUT_SECONDS:-0}"
CASES="${CASES:-native-arm64,amd64-emulated}"

MESSAGE="${MESSAGE:-}"
ADDRESS="${ADDRESS:-}"
SIGNATURE="${SIGNATURE:-}"

usage() {
    cat <<EOF
Usage: $0 [info|dev-prove|prove]

Environment:
  NATIVE_IMAGE       Native ARM64 image to benchmark (default: $NATIVE_IMAGE)
  AMD64_IMAGE        AMD64 image tag to benchmark under emulation (default: $AMD64_IMAGE)
  PUBLIC_IMAGE       Remote image to pull when PREPARE_AMD64=1 (default: $PUBLIC_IMAGE)
  PREPARE_AMD64=1    Pull PUBLIC_IMAGE as linux/amd64 and retag it as AMD64_IMAGE
  RESULTS_DIR        Directory for logs and summary files
  STAT_INTERVAL      Seconds between docker stats samples (default: $STAT_INTERVAL)
  TIMEOUT_SECONDS    Stop a run after this many seconds; 0 disables timeout
  CASES              Comma-separated cases: native-arm64,amd64-emulated (default: $CASES)
  MESSAGE            Proof message; required for dev-prove and prove
  ADDRESS            Bitcoin address; required for dev-prove and prove
  SIGNATURE          Bitcoin message signature; required for dev-prove and prove

Modes:
  info        Run 'og-zkp info --json'
  dev-prove   Run proof flow with RISC0_DEV_MODE=1; fast but not a real proof
  prove       Run real Groth16 proving; expensive, use TIMEOUT_SECONDS for emulation
EOF
}

if [[ "$MODE" == "-h" || "$MODE" == "--help" ]]; then
    usage
    exit 0
fi

case "$MODE" in
    info|dev-prove|prove) ;;
    *)
        usage >&2
        exit 1
        ;;
esac

if [[ "$MODE" != "info" && (-z "$MESSAGE" || -z "$ADDRESS" || -z "$SIGNATURE") ]]; then
    echo "MESSAGE, ADDRESS, and SIGNATURE are required for $MODE mode." >&2
    usage >&2
    exit 1
fi

mkdir -p "$RESULTS_DIR"

if [[ "${PREPARE_AMD64:-0}" == "1" ]]; then
    docker pull --platform linux/amd64 "$PUBLIC_IMAGE"
    docker tag "$PUBLIC_IMAGE" "$AMD64_IMAGE"
    if docker image inspect "$NATIVE_IMAGE" >/dev/null 2>&1; then
        docker tag "$NATIVE_IMAGE" "$PUBLIC_IMAGE"
    fi
fi

require_image() {
    local image="$1"
    if ! docker image inspect "$image" >/dev/null 2>&1; then
        echo "Missing image: $image" >&2
        if [[ "$image" == "$AMD64_IMAGE" ]]; then
            echo "Run with PREPARE_AMD64=1 to create the amd64 baseline tag." >&2
        fi
        exit 1
    fi
}

image_arch() {
    local image="$1"
    docker image inspect "$image" --format '{{.Architecture}}/{{.Os}}' 2>/dev/null || echo missing
}

write_metadata() {
    {
        echo "mode=$MODE"
        echo "date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "native_image=$NATIVE_IMAGE"
        echo "amd64_image=$AMD64_IMAGE"
        echo "stat_interval=$STAT_INTERVAL"
        echo "timeout_seconds=$TIMEOUT_SECONDS"
        echo "cases=$CASES"
        echo "host_uname=$(uname -a)"
        if command -v sysctl >/dev/null 2>&1; then
            echo "host_cpu=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || true)"
            echo "host_ncpu=$(sysctl -n hw.ncpu 2>/dev/null || true)"
            echo "host_mem_bytes=$(sysctl -n hw.memsize 2>/dev/null || true)"
        fi
        echo "docker_info=$(docker info --format 'server={{.ServerVersion}} arch={{.Architecture}} os={{.OSType}} cpus={{.NCPU}} mem={{.MemTotal}}')"
        echo "native_image_arch=$(image_arch "$NATIVE_IMAGE")"
        echo "amd64_image_arch=$(image_arch "$AMD64_IMAGE")"
    } > "$RESULTS_DIR/metadata.txt"
}

run_case() {
    local name="$1"
    local platform="$2"
    local image="$3"
    local container="og-zkp-bench-${name}-$$"
    local log="$RESULTS_DIR/$name.log"
    local stats="$RESULTS_DIR/$name.stats.csv"
    local inspect="$RESULTS_DIR/$name.inspect.json"
    local summary="$RESULTS_DIR/summary.tsv"
    local -a env_args=()
    local -a cmd_args=()

    case "$MODE" in
        info)
            cmd_args=(info --json)
            ;;
        dev-prove)
            env_args=(-e RISC0_DEV_MODE=1)
            cmd_args=(prove --message "$MESSAGE" --address "$ADDRESS" --signature "$SIGNATURE")
            ;;
        prove)
            cmd_args=(prove --message "$MESSAGE" --address "$ADDRESS" --signature "$SIGNATURE")
            ;;
    esac

    echo "Running $name ($platform, $image, mode=$MODE)"

    printf 'elapsed_seconds,cpu_percent,mem_usage,mem_percent,net_io,block_io,pids\n' > "$stats"

    local start now elapsed status timed_out exit_code oom
    start="$(date +%s)"
    if [[ "${#env_args[@]}" -gt 0 ]]; then
        docker run -d --name "$container" --platform "$platform" "${env_args[@]}" "$image" "${cmd_args[@]}" >/dev/null
    else
        docker run -d --name "$container" --platform "$platform" "$image" "${cmd_args[@]}" >/dev/null
    fi

    timed_out=0
    while true; do
        status="$(docker inspect --format '{{.State.Running}}' "$container" 2>/dev/null || true)"
        now="$(date +%s)"
        elapsed=$((now - start))

        if [[ "$status" != "true" ]]; then
            break
        fi

        docker stats --no-stream --format '{{.CPUPerc}},{{.MemUsage}},{{.MemPerc}},{{.NetIO}},{{.BlockIO}},{{.PIDs}}' "$container" \
            | while IFS= read -r line; do printf '%s,%s\n' "$elapsed" "$line"; done >> "$stats"

        if [[ "$TIMEOUT_SECONDS" != "0" && "$elapsed" -ge "$TIMEOUT_SECONDS" ]]; then
            timed_out=1
            docker stop "$container" >/dev/null || true
            break
        fi

        sleep "$STAT_INTERVAL"
    done

    now="$(date +%s)"
    elapsed=$((now - start))

    docker logs "$container" > "$log" 2>&1 || true
    docker inspect "$container" > "$inspect" 2>/dev/null || true

    exit_code="$(docker inspect --format '{{.State.ExitCode}}' "$container" 2>/dev/null || echo unknown)"
    oom="$(docker inspect --format '{{.State.OOMKilled}}' "$container" 2>/dev/null || echo unknown)"
    docker rm "$container" >/dev/null 2>&1 || true

    if [[ ! -f "$summary" ]]; then
        printf 'name\tplatform\timage\tmode\twall_seconds\texit_code\toom\ttimed_out\n' > "$summary"
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$name" "$platform" "$image" "$MODE" "$elapsed" "$exit_code" "$oom" "$timed_out" >> "$summary"
}

write_metadata

should_run() {
    case ",$CASES," in
        *",$1,"*) return 0 ;;
        *)         return 1 ;;
    esac
}

if should_run native-arm64; then
    require_image "$NATIVE_IMAGE"
    run_case native-arm64 linux/arm64 "$NATIVE_IMAGE"
fi

if should_run amd64-emulated; then
    require_image "$AMD64_IMAGE"
    run_case amd64-emulated linux/amd64 "$AMD64_IMAGE"
fi

echo "Wrote benchmark results to $RESULTS_DIR"
