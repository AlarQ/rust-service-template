#!/bin/bash

set -e

# Get the script's directory (works regardless of where script is called from)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_DIR="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$SCRIPT_DIR/git-hooks"
GIT_HOOKS_DIR="$(git rev-parse --show-toplevel)/.git/hooks"

echo "Installing git hooks for rust-service-template..."

# Create .git/hooks directory if it doesn't exist
mkdir -p "$GIT_HOOKS_DIR"

# Install pre-push hook
cp "$HOOKS_DIR/pre-push" "$GIT_HOOKS_DIR/pre-push"
chmod +x "$GIT_HOOKS_DIR/pre-push"

echo "Pre-push hook installed"

# Check for required tools
echo ""
echo "Checking required tools..."

if command -v cargo-audit &> /dev/null; then
    echo "cargo-audit installed"
else
    echo "WARNING: cargo-audit not installed (optional)"
    echo "   Install with: cargo install cargo-audit"
fi

if command -v cargo-outdated &> /dev/null; then
    echo "cargo-outdated installed"
else
    echo "WARNING: cargo-outdated not installed (optional)"
    echo "   Install with: cargo install cargo-outdated"
fi

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "The pre-push hook will run:"
echo "  - cargo +nightly fmt (auto-format and check)"
echo "  - cargo clippy"
echo "  - cargo test"
echo "  - cargo audit (if installed)"
echo "  - cargo outdated (if installed)"
