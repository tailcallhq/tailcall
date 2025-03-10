#!/bin/bash

set -e

# Get version argument from command line
VERSION=${1:-"latest"}

BASE_URL="https://github.com/tailcallhq/tailcall/releases/download"

if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl --silent "https://api.github.com/repos/tailcallhq/tailcall/releases/latest" | jq -r '.tag_name')
fi

# Determine OS and architecture to get the correct URL
if [[ "$OSTYPE" == "darwin"* ]]; then
  OS="apple-darwin"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
  OS="unknown-linux-gnu"
elif [[ "$OSTYPE" == "linux-musl"* ]]; then
  OS="unknown-linux-musl"
else
  OS="unknown"
fi

if [[ "$(uname -m)" == "x86_64" ]]; then
  ARCH="x86_64"
elif [[ "$(uname -m)" == "i686" ]]; then
  ARCH="i686"
elif [[ "$(uname -m)" == "aarch64" ]] || [[ "$(uname -m)" == "arm64" ]]; then
  ARCH="aarch64"
else
  ARCH="unknown"
fi

# Ensure we recognize the OS and architecture
if [[ "$OS" == "unknown" ]] || [[ "$ARCH" == "unknown" ]]; then
  echo "Unsupported OS or architecture."
  exit 1
fi

# Derive download URL based on detected OS and architecture
URL="$BASE_URL/$VERSION/tailcall-${ARCH}-${OS}"

# Prepare versioned directory for download
INSTALL_DIR="$HOME/.tailcall/lib/$VERSION"
mkdir -p "$INSTALL_DIR"

# Download the executable directly into the versioned directory
curl -#Lo "$INSTALL_DIR/tailcall-${OS}-${ARCH}" "$URL"
chmod +x "$INSTALL_DIR/tailcall-${OS}-${ARCH}"

# Create symlinks in ~/.tailcall/bin
mkdir -p "$HOME/.tailcall/bin"
ln -sf "$INSTALL_DIR/tailcall-${OS}-${ARCH}" "$HOME/.tailcall/bin/tailcall"

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
