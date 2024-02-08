#!/bin/bash

# Flag to keep track of when we're inside the desired section
inside_section=false

# Read the file line by line
while IFS= read -r line; do
    # Check if we've reached the tooling-version section
    if [[ "$line" == "[package.metadata.tooling-version]" ]]; then
        inside_section=true
        continue
    fi

    # Check if we've reached the next section (denoted by [])
    if [[ "$inside_section" == true && "$line" =~ ^\[.*\]$ ]]; then
        break
    fi

    # If we're inside the section, extract key and value
    if [[ "$inside_section" == true && "$line" != "" ]]; then
        # Extract key and value using bash string manipulation
        key=$(echo "$line" | cut -d '=' -f 1 | xargs)
        value=$(echo "$line" | cut -d '=' -f 2 | xargs | tr -d '"')

        # Add _version suffix to the key
        key="${key}_expected_version"

        # Assign to variables with _expected_version suffix and print (for demonstration)
        declare "$key=$value"
        echo "$key is locked to '${!key}'"
    fi
done < "Cargo.toml"

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

match_lib_version() {
    actual=$1
    lib=$2
    var_name="${lib}_expected_version"
    expected="${!var_name}"

    # Check if the resolved version contains the expected version string
    if [[ "$actual" != "$expected"* ]]; then
        echo "Expected $lib version: $expected, found: $actual"
        exit 1
    fi
}

run_cargo_clippy() {
    MODE=$1

    match_lib_version "$(cargo clippy --version)" "clippy"

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

    match_lib_version "$(prettier --version)" "prettier"

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

run_testconv() {
    MODE=$1
    if [ "$MODE" == "fix" ]; then
        cargo run -p testconv
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

        run_testconv $MODE
        TESTCONV_EXIT_CODE=$?
        run_prettier $MODE
        PRETTIER_EXIT_CODE=$?
        ;;
    *)
        echo "Invalid mode. Please use --mode=check or --mode=fix"
        exit 1
        ;;
esac

# If any command failed, exit with a non-zero status code
if [ $FMT_EXIT_CODE -ne 0 ] || [ $CLIPPY_EXIT_CODE -ne 0 ] || [ $PRETTIER_EXIT_CODE -ne 0 ] || [ $AUTOGEN_SCHEMA_EXIT_CODE -ne 0 ] || [ $TESTCONV_EXIT_CODE -ne 0 ]; then
    exit 1
fi