#!/usr/bin/env bash
set -euo pipefail

# Multi-architecture build script for local development
# Usage: ./build-multiarch.sh [version]

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
VERSION="${1:-local}"
BASE_PLUGIN_NAME="nomadicdrones/rustycan4docker"

# Detect current architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        DOCKER_ARCH="amd64"
        ;;
    aarch64)
        DOCKER_ARCH="arm64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "Building plugin for architecture: $DOCKER_ARCH"
echo "Version: $VERSION"

# Set plugin name with architecture suffix
export PLUGIN_NAME="${BASE_PLUGIN_NAME}-${DOCKER_ARCH}:${VERSION}"

# Build the plugin
cd "${SCRIPT_DIR}"
chmod +x build-plugin.sh
sudo ./build-plugin.sh

echo ""
echo "✅ Plugin built successfully!"
echo "Plugin name: $PLUGIN_NAME"
echo ""
echo "To enable the plugin:"
echo "  docker plugin enable $PLUGIN_NAME"
echo ""
echo "To test the plugin:"
echo "  docker network create -d $PLUGIN_NAME test-can-network"