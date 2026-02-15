#!/bin/bash
# Shim that replaces the Docker CLI for Groth16 proving.
# risc0-groth16 shells out to `docker run` to invoke the prover.
# This script intercepts that call and runs the prover directly.

case "$1" in
    --version)
        echo "groth16-shim (local prover)"
        exit 0
        ;;
    run)
        # Parse: docker run --rm -v {work_dir}:/mnt {image}
        shift
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -v)
                    SRC="${2%%:*}"
                    shift 2
                    ;;
                --rm)
                    shift
                    ;;
                *)
                    shift
                    ;;
            esac
        done
        rm -rf /mnt
        ln -sfn "$SRC" /mnt
        cd /app
        exec /app/prover.sh
        ;;
    *)
        exit 1
        ;;
esac
