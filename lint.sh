#!/bin/bash

# Configuration for file types to be tested via prettier
FILE_TYPES="{graphql,yml,json,md,ts,js}"

run_cargo_fmt() {
    MODE=$1
    if [ "$MODE" == "check" ]; then
        cargo fmt --all -- --check
    else
        cargo fmt --all
    fi
    return $?
}

run_cargo_clippy() {
    MODE=$1
    CMD="cargo clippy --all --all-targets --all-features"
    if [ "$MODE" == "fix" ]; then
        $CMD --fix --allow-staged --allow-dirty
    fi
    CMD="$CMD -- -D warnings"
    $CMD
    return $?
}

run_prettier() {
    MODE=$1
    if [ "$MODE" == "check" ]; then
        prettier -c .prettierrc --check "**/*.$FILE_TYPES"
    else
        prettier -c .prettierrc --write "**/*.$FILE_TYPES"
    fi
    return $?
}

run_autogen_schema() {
    MODE=$1
    cargo run -p autogen $MODE
    return $?
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
        run_autogen_schema $MODE
        AUTOGEN_SCHEMA_EXIT_CODE=$?

        # Commands that uses nightly toolchains are run from `.nightly` directory
        # to read the nightly version from `rust-toolchain.toml` file
        pushd .nightly
        run_cargo_fmt $MODE
        FMT_EXIT_CODE=$?
        run_cargo_clippy $MODE
        CLIPPY_EXIT_CODE=$?
        popd

        run_prettier $MODE
        PRETTIER_EXIT_CODE=$?
        ;;
    *)
        echo "Invalid mode. Please use --mode=check or --mode=fix"
        exit 1
        ;;
esac

# If any command failed, exit with a non-zero status code
if [ $FMT_EXIT_CODE -ne 0 ] || [ $CLIPPY_EXIT_CODE -ne 0 ] || [ $PRETTIER_EXIT_CODE -ne 0 ] || [ $AUTOGEN_SCHEMA_EXIT_CODE -ne 0 ]; then
    exit 1
fi