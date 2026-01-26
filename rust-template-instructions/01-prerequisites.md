# Prerequisites & Git Hooks Configuration

[← Overview](00-overview.md) | [Next: Code Quality Config →](02-code-quality-config.md)

---

## Required Tools

Install these tools before setting up the project:

```bash
# Required: Nightly toolchain for rustfmt
rustup toolchain install nightly

# Required: Security audit
cargo install cargo-audit

# Required: Dependency updates checker
cargo install cargo-outdated
```

---

## Pre-push Hook

Create `scripts/git-hooks/pre-push`:

```bash
#!/bin/bash

set -e

echo "Running pre-push checks..."

# Get the repo root
REPO_ROOT=$(git rev-parse --show-toplevel)

cd "$REPO_ROOT"

# 1. Check formatting
echo "Checking rustfmt..."
if ! cargo +nightly fmt --check; then
    echo "Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi
echo "rustfmt passed"

# 2. Run clippy
echo "Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "Clippy found issues. Fix them before pushing."
    exit 1
fi
echo "clippy passed"

# 3. Run tests
echo "Running tests..."
if ! cargo test --all-features; then
    echo "Tests failed. Fix them before pushing."
    exit 1
fi
echo "tests passed"

# 4. Run cargo audit
echo "Running cargo audit..."
if ! cargo audit; then
    echo "Security vulnerabilities found. Review before pushing."
    exit 1
fi
echo "cargo audit passed"

# 5. Run cargo outdated
echo "Checking outdated dependencies..."
cargo outdated || echo "Some dependencies are outdated"

echo "All pre-push checks passed!"
```

---

## Install Script

Create `scripts/install-hooks.sh`:

```bash
#!/bin/bash

set -e

# Get the script's directory (works regardless of where script is called from)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_DIR="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$SCRIPT_DIR/git-hooks"
GIT_HOOKS_DIR="$(git rev-parse --show-toplevel)/.git/hooks"

echo "Installing git hooks for {service-name}..."

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
    echo "ERROR: cargo-audit not installed"
    echo "   Install with: cargo install cargo-audit"
    exit 1
fi

if command -v cargo-outdated &> /dev/null; then
    echo "cargo-outdated installed"
else
    echo "ERROR: cargo-outdated not installed"
    echo "   Install with: cargo install cargo-outdated"
    exit 1
fi

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "The pre-push hook will run:"
echo "  - cargo +nightly fmt"
echo "  - cargo clippy"
echo "  - cargo test"
echo "  - cargo audit"
echo "  - cargo outdated"
```

---

## Installation

```bash
# Install required tools first
cargo install cargo-audit cargo-outdated

# Run the install script
./scripts/install-hooks.sh
```

---

## Directory Structure

```
scripts/
├── install-hooks.sh      # Hook installation script
└── git-hooks/
    └── pre-push          # Pre-push hook
```

---

[← Overview](00-overview.md) | [Next: Code Quality Config →](02-code-quality-config.md)
