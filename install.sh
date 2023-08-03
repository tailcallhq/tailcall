#!/bin/bash

set -e

# Get version argument from command line
VERSION=${1:-"latest"}

BASE_URL="https://github.com/tailcallhq/tailcall/releases/download"

if [ "$VERSION" = "latest" ]; then
  # Fetch latest version from local JSON file
  VERSION=$(jq -r '.version' version.json)
fi

# Derive download URL
URL="$BASE_URL/$VERSION/tailcall-$VERSION.zip"

# Prepare versioned directory for download
INSTALL_DIR="$HOME/.tailcall/lib/$VERSION"
mkdir -p "$INSTALL_DIR"

# Download and extract the zip file into versioned directory
curl -#L "$URL" -o "$INSTALL_DIR/tailcall.zip"
unzip -o "$INSTALL_DIR/tailcall.zip" -d "$INSTALL_DIR"
rm "$INSTALL_DIR/tailcall.zip"

# Create symlinks in ~/.tailcall/bin
mkdir -p "$HOME/.tailcall/bin"
ln -sf "$INSTALL_DIR/bin/tailcall_cli_main" "$HOME/.tailcall/bin/tc"

# Determine which shell the user is running and which profile file to update
if [[ "$SHELL" == *"zsh"* ]]; then
  SHELL_PROFILE="$HOME/.zshrc"
elif [[ "$SHELL" == *"bash"* ]]; then
  if [ -f "$HOME/.bash_profile" ]; then
    SHELL_PROFILE="$HOME/.bash_profile"
  else
    SHELL_PROFILE="$HOME/.bashrc"
  fi
fi

# Provide instructions to add ~/.tailcall/bin to PATH in shell profile
echo "Installation complete. Please add the following line to your $SHELL_PROFILE:"
echo 'export PATH="$HOME/.tailcall/bin:$PATH"'
