#!/bin/bash

CARGO_TOML_PATH="../node/Cargo.toml"

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 [compile|publish]"
    exit 1
fi

if [ ! -f "$CARGO_TOML_PATH" ]; then
    echo "Cargo.toml not found, exiting."
    exit 1
fi

# Rename wasm-node to wasm-browser
sed -i '' 's/name = "wasm-node"/name = "wasm-browser"/' "$CARGO_TOML_PATH"

cargo install -q worker-build

case $1 in
    compile)
        # Run wasm-pack build command
        wasm-pack build ../node --target web --scope tailcallhq --no-typescript --out-name wasm_browser --out-dir ../browser/pkg
        ;;
    publish)
        # Run wasm-pack publish command
        wasm-pack publish ../node --target web
        ;;
    *)
        sed -i '' 's/name = "wasm-browser"/name = "wasm-node"/' "$CARGO_TOML_PATH"
        echo "Invalid argument. Use 'compile' or 'publish'."
        exit 1
        ;;
esac

# Check if wasm-pack publish was successful
if [ $? -ne 0 ]; then
    echo "wasm-pack command failed."
fi

# Rename wasm-browser back to wasm-node
sed -i '' 's/name = "wasm-browser"/name = "wasm-node"/' "$CARGO_TOML_PATH"
