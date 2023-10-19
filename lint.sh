#!/bin/bash

# Configuration for file types to be tested via prettier
FILE_TYPES="{graphql,yml,json,md}"

run_cargo_fmt() {
    MODE=$1
    if [ "$MODE" == "check" ]; then
        cargo +nightly fmt -- --check
    else
        cargo +nightly fmt
    fi
}

run_cargo_clippy() {
    MODE=$1
    CMD="cargo +nightly clippy --all-targets --all-features"
    [ "$MODE" == "check" ] || CMD="$CMD --fix --allow-staged --allow-dirty"
    CMD="$CMD -- -D warnings"
    $CMD
}

run_prettier() {
    MODE=$1
    if [ "$MODE" == "check" ]; then
        prettier --check "**/*.$FILE_TYPES"
    else
        prettier --write "**/*.$FILE_TYPES"
    fi
}

# Extract the mode from the argument
if [[ $1 == "--mode="* ]]; then
    MODE=${1#--mode=}
else
    echo "Please specify a mode with --mode=check or --mode=fix"
    exit 1
fi

# Run commands based on mode
case $MODE in
    check|fix)
        run_cargo_fmt $MODE
        run_cargo_clippy $MODE
        run_prettier $MODE
        ;;
    *)
        echo "Invalid mode. Please use --mode=check or --mode=fix"
        exit 1
        ;;
esac
