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

# Optional: Dependency updates checker
cargo install cargo-outdated
```

---

## Git Workflows and CI/CD

This repository includes comprehensive GitHub Actions workflows for:

- **Conventional commits validation** - Enforces structured commit messages on PRs
- **Automated CI** - Full pipeline with tests, linting, builds, and security checks
- **Semantic versioning** - Automatic version bumping based on commit types
- **Automated releases** - Creates GitHub releases with changelogs and binaries
- **Changelog generation** - Automatic CHANGELOG.md updates using git-cliff

**See [Git Workflows](27-git-workflows.md) for complete CI/CD documentation.**

---

## Pre-push Hook

The pre-push hook runs local quality checks before pushing to remote.

**Location:** `.git/hooks/pre-push`

**Checks Performed:**
1. **Formatting** - `cargo +nightly fmt` (auto-formats and fails if changes made)
2. **Clippy** - `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests** - `cargo test --all-features`
4. **Security Audit** - `cargo audit` (if installed, fails on vulnerabilities)
5. **Outdated Dependencies** - `cargo outdated` (warning only, if installed)

**Hook Implementation:**

```bash
#!/bin/bash

set -e

echo "🔧 Running pre-push checks..."

# Get the repo root (transaction-service is its own repo)
REPO_ROOT=$(git rev-parse --show-toplevel)

cd "$REPO_ROOT"

# 1. Check formatting (auto-formats and fails if changes were made)
echo "📝 Checking rustfmt..."
if ! cargo +nightly fmt; then
    echo "❌ Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi
echo "✅ rustfmt passed"

# 2. Run clippy
echo "🔍 Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy found issues. Fix them before pushing."
    exit 1
fi
echo "✅ clippy passed"

# 3. Run tests
echo "🧪 Running tests..."
if ! cargo test --all-features; then
    echo "❌ Tests failed. Fix them before pushing."
    exit 1
fi
echo "✅ tests passed"

# 4. Run cargo audit (if installed)
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Running cargo audit..."
    if ! cargo audit; then
        echo "⚠️  Security vulnerabilities found. Review before pushing."
        exit 1
    fi
    echo "✅ cargo audit passed"
else
    echo "⏭️  cargo-audit not installed, skipping (install with: cargo install cargo-audit)"
fi

# 5. Check outdated dependencies (if installed) - warning only
if command -v cargo-outdated &> /dev/null; then
    echo "📦 Checking outdated dependencies..."
    OUTDATED=$(cargo outdated --root-deps-only --format json 2>/dev/null | jq '.dependencies | length' 2>/dev/null || echo "0")
    if [ "$OUTDATED" != "0" ] && [ "$OUTDATED" != "" ]; then
        echo "⚠️  $OUTDATED outdated dependencies found. Run 'cargo outdated' for details."
    else
        echo "✅ dependencies up to date"
    fi
else
    echo "⏭️  cargo-outdated not installed, skipping (install with: cargo install cargo-outdated)"
fi

echo "🎉 All pre-push checks passed!"
```

---

## Skipping Pre-push Checks

To bypass the pre-push hook (not recommended):

```bash
git push --no-verify
```

**Use cases:**
- Emergency fixes when CI will catch issues
- Force-pushing to protected branches
- Temporary work-in-progress branches

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
```

---

## Installation

```bash
# Install required tools first
rustup toolchain install nightly
cargo install cargo-audit cargo-outdated

# Run the install script
./scripts/install-hooks.sh
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
.git/
└── hooks/
    └── pre-push          # Pre-push hook (installed to .git/hooks/)

scripts/
├── install-hooks.sh      # Hook installation script
└── git-hooks/
    └── pre-push          # Pre-push hook template

.github/
├── workflows/
│   ├── commits.yml       # Conventional commits validation
│   ├── ci.yml            # CI pipeline
│   └── release.yml       # Automated releases
└── actions/
    ├── setup-rust.yml    # Rust toolchain setup
    ├── setup-sqlx.yml    # SQLx CLI and migrations
    ├── setup-git-cliff.yml # git-cliff installation
    ├── version-bump.yml  # Semantic version calculation
    └── update-changelog.yml # CHANGELOG.md generation

.cliff.toml               # git-cliff configuration
CHANGELOG.md              # Generated changelog
```

---

## Next Steps

- **For CI/CD details:** See [Git Workflows](27-git-workflows.md)
- **For code quality configuration:** See [Code Quality Config](02-code-quality-config.md)
- **For repository setup:** Continue to [Project Structure](03-project-structure.md)

---

[← Overview](00-overview.md) | [Next: Code Quality Config →](02-code-quality-config.md)
