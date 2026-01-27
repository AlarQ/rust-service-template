# Git Workflows and CI/CD

[← Prerequisites](01-prerequisites.md) | [Next: Quick Start Checklist →](26-quick-start-checklist.md)

---

## Overview

This repository uses a sophisticated CI/CD pipeline with GitHub Actions that includes:

- **Conventional commits validation** - Enforces structured commit messages
- **Automated CI** - Tests, linting, builds, and security checks
- **Semantic versioning** - Automatic version bumping based on commit types
- **Automated releases** - Creates GitHub releases with changelogs
- **Changelog generation** - Automatic CHANGELOG.md updates using git-cliff

---

## GitHub Actions Structure

```
.github/
├── workflows/
│   ├── commits.yml       # Conventional commits validation
│   ├── ci.yml            # Full CI pipeline
│   └── release.yml       # Automated releases
└── actions/
    ├── setup-rust.yml     # Rust toolchain installer
    ├── setup-sqlx.yml     # SQLx CLI and migrations
    ├── setup-git-cliff.yml # git-cliff installation
    ├── version-bump.yml   # Semantic version calculation
    └── update-changelog.yml # CHANGELOG.md generation
```

---

## 1. Conventional Commits Validation

**File:** `.github/workflows/commits.yml`

### Purpose

Enforces conventional commit format on pull requests to ensure consistent commit history and automated versioning.

### Trigger

Runs on PR events:
- `opened`
- `synchronize` (new commits pushed)
- `reopened`

### Conventional Commit Format

```
<type>[!][(scope)]: <description>

Examples:
feat(api): add transaction pagination
fix!: breaking change to transaction model
docs: update API documentation
chore(deps): update reqwest to latest version
```

### Commit Types

#### Version Bumping Types (visible in changelog):
- `feat` - New features (minor version bump)
- `fix` - Bug fixes (patch version bump)
- `perf` - Performance improvements (patch version bump)
- `refactor` - Code refactoring (patch version bump)
- `docs` - Documentation changes
- `security` - Security fixes

#### Maintenance Types (hidden in changelog):
- `style` - Code style changes (formatting, etc.)
- `test` - Adding or updating tests
- `build` - Build system changes
- `ci` - CI/CD changes
- `chore` - Maintenance tasks
- `deps` - Dependency updates
- `release` - Release automation
- `revert` - Reverting previous changes

### Breaking Changes

Add `!` after the type to indicate breaking changes:
```
feat!(api): remove deprecated endpoints
fix!(domain): change transaction id format
```

Breaking changes trigger **major** version bumps.

### Validation Workflow

```yaml
- Uses setup-git-cliff action
- Gets commits in PR: `git log origin/main..HEAD`
- Validates each commit against regex pattern
- Fails PR if any commit is invalid
```

### Regex Pattern

```
^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert|deps|security|release)(!?)?(\(.+\))?: .+
```

### Error Output

If commits are invalid, the workflow shows:
- ❌ List of invalid commits
- ✅ Expected format
- 📋 All commit types with descriptions
- 💡 Examples of valid commits

---

## 2. Continuous Integration (CI)

**File:** `.github/workflows/ci.yml`

### Purpose

Full CI pipeline ensuring code quality, functionality, and security.

### Trigger

Runs on:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`

### Jobs

#### Test Suite (`test`)

**Services:**
- PostgreSQL 15 on port 5433
- Kafka 7.4.0 on port 9092

**Steps:**
1. Checkout repository
2. Setup Rust toolchain
3. Cache cargo registry and target
4. Setup SQLx and run migrations
5. Run tests: `cargo test --all-features`

**Environment Variables:**
```yaml
DATABASE_URL: postgresql://postgres:postgres@localhost:5433/postgres?schema=transaction_service
KAFKA_BOOTSTRAP_SERVERS: localhost:9092
OPENAI_API_KEY: sk-test-key
JWT_SECRET: test-secret
```

#### Code Quality (`lint`)

**Steps:**
1. Checkout repository
2. Setup Rust with clippy component
3. Install nightly toolchain with rustfmt
4. Cache cargo registry
5. Check formatting: `cargo +nightly fmt -- --check`
6. Run clippy: `cargo clippy --all-features --offline -- -D warnings`

**Rules:**
- Any clippy warning fails the build (`-D warnings`)
- Formatting must match nightly rustfmt

#### Build (`build`)

**Dependencies:** Runs after `test` and `lint` jobs pass

**Steps:**
1. Checkout repository
2. Setup Rust toolchain
3. Cache cargo registry
4. Build release: `cargo build --release`

#### Security Audit (`security`)

**Steps:**
1. Checkout repository
2. Setup Rust toolchain
3. Install cargo-audit
4. Run audit: `cargo audit`

**Purpose:** Detects security vulnerabilities in dependencies using the RustSec Advisory Database.

### Caching Strategy

All jobs cache:
- `~/.cargo/registry` - Cargo registry index
- `~/.cargo/git` - Git dependencies
- `target` - Build artifacts

Cache key: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`

---

## 3. Automated Releases

**File:** `.github/workflows/release.yml`

### Purpose

Automated semantic versioning and release creation with changelogs.

### Triggers

#### Manual Trigger
- `workflow_dispatch` with optional `force_bump` parameter
- Options: `patch`, `minor`, `major`, or empty (automatic)

#### Automatic Trigger
- Runs after `CI` workflow completes successfully
- Only on `main` branch pushes

### Version Bump Logic

**Action:** `.github/actions/version-bump.yml`

The version bump is calculated based on conventional commits since the last tag:

| Commit Type Found | Version Bump |
|-------------------|--------------|
| `feat!`, `fix!`, etc. (any type with `!`) | **major** |
| `feat` | **minor** |
| `fix`, `perf`, `refactor`, `revert`, `security` | **patch** |
| `deps`, `release` | **patch** |
| None of the above | **patch** (default) |

### Version Calculation

1. Read current version from `Cargo.toml` (source of truth)
2. Get commits since last tag (or all commits if no tag)
3. Determine bump type based on commit types
4. Calculate new version: `MAJOR.MINOR.PATCH`
5. Check if tag already exists (fails if duplicate)
6. Output: `bump-type`, `current-version`, `new-version`, `new-tag`

### Release Process

1. **Checkout repository** with full git history
2. **Setup Rust toolchain**
3. **Cache cargo registry**
4. **Setup git-cliff** for changelog generation
5. **Calculate version bump** using conventional commits
6. **Update Cargo.toml** with new version
7. **Update CHANGELOG.md** with new version section
8. **Build release binary**: `cargo build --release`
9. **Create release archive**:
   - Includes: binary, `run.sh`, `README.md`, `CHANGELOG.md`
   - Filename: `transaction-service-{tag}-linux-x86_64.tar.gz`
10. **Commit version and changelog**:
    - Commit message: `chore(release): {tag}`
    - Create annotated tag: `git tag -a "{tag}"`
    - Push to `main` and push tag
11. **Create GitHub Release**:
    - Tag name: `v{version}`
    - Release name: `Release v{version}`
    - Body: Generated by git-cliff with installation instructions
    - Attach: release archive
12. **Update latest tag**: `git tag -f latest && git push -f origin latest`

### GitHub Release Body

Generated automatically with:

```markdown
## Release {tag}

### Changes
<git-cliff generated changelog>

### Installation
```bash
# Download the release
wget https://github.com/AlarQ/transaction-service/releases/download/{tag}/transaction-service-{tag}-linux-x86_64.tar.gz

# Extract
tar -xzf transaction-service-{tag}-linux-x86_64.tar.gz

# Run
./run.sh
```

### Docker
```bash
# Build image
docker build -t transaction-service:{tag} .

# Run with dependencies
docker-compose up -d
docker run -p 8081:8081 transaction-service:{tag}
```
```

### Permissions

Required permissions in `release.yml`:
```yaml
permissions:
  contents: write    # Create tags and releases
  pull-requests: write  # Update PRs (if needed)
  packages: write    # Publish to GitHub Packages (if used)
```

---

## 4. Changelog Generation

### git-cliff Configuration

**File:** `.cliff.toml`

git-cliff is a command-line tool for generating changelogs from Git history.

### Features

- Parses conventional commits
- Groups commits by type
- Supports breaking changes
- Generates semantic versioned changelogs
- Customizable templates

### Configuration Structure

```toml
[changelog]
# Changelog header and footer templates
header = "# Changelog\n"
body = """
## [{{ version | replace_start:"v:" }}] - {{ date | date_str(format="%Y-%m-%d") }}
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group }}
{% for commit in commits %}
- {{ commit.message | upper_first }}
{% endfor %}
{% endfor %}
"""
trim = true

[git]
# Commit parsing
commit_parsers = [
  { message = "^feat", group = "Features" },
  { message = "^fix", group = "Bug Fixes" },
  { message = "^docs", group = "Documentation" },
  # ... more parsers
]
# Commit filtering
filter_commits = false
# Tag pattern
tag_pattern = "v[0-9]+.*"
# Sort commits
sort_commits = "oldest"
```

### Update Changelog Action

**File:** `.github/actions/update-changelog.yml`

**Steps:**
1. Check if `CHANGELOG.md` exists
2. Create temporary file with new version section
3. Generate changelog using git-cliff for latest tag
4. Append existing changelog content (preserve history)
5. Move temp file to `CHANGELOG.md`

**Example Output:**

```markdown
# Changelog

## [1.2.0] - 2026-01-26

### Features
- feat(api): add transaction pagination
- feat(ai): implement batch categorization

### Bug Fixes
- fix: handle transaction pagination edge cases
- fix(api): correct JWT validation error messages

### Documentation
- docs: update API documentation for endpoints
```

---

## 5. Custom GitHub Actions

### Setup Rust Action

**File:** `.github/actions/setup-rust.yml`

Installs Rust stable toolchain with optional components.

**Inputs:**
- `components` - Comma-separated list (e.g., `rustfmt,clippy`)

**Usage:**
```yaml
- uses: ./.github/actions/setup-rust
  with:
    components: clippy
```

### Setup SQLx Action

**File:** `.github/actions/setup-sqlx.yml`

Installs sqlx-cli and runs database migrations.

**Inputs:**
- `database_url` - PostgreSQL connection URL

**Steps:**
1. Cache sqlx-cli binary
2. Install if not cached: `cargo install sqlx-cli --no-default-features --features postgres`
3. Run migrations: `sqlx migrate run --database-url "{database_url}"`

**Usage:**
```yaml
- uses: ./.github/actions/setup-sqlx
  with:
    database_url: postgresql://postgres:postgres@localhost:5433/postgres?schema=transaction_service
```

### Setup git-cliff Action

**File:** `.github/actions/setup-git-cliff.yml`

Installs git-cliff for changelog generation.

**Inputs:**
- `version` - git-cliff version (default: `2.12.0`)

**Steps:**
1. Download git-cliff release tar.gz from GitHub
2. Extract and install binary to `/usr/local/bin/`
3. Verify installation with `git-cliff --version`

**Usage:**
```yaml
- uses: ./.github/actions/setup-git-cliff
  with:
    version: "2.12.0"
```

### Version Bump Action

**File:** `.github/actions/version-bump.yml`

Calculates semantic version bump based on conventional commits.

**Inputs:**
- `force_bump` - Override automatic detection (`patch`, `minor`, `major`, or empty)

**Outputs:**
- `bump-type` - Calculated bump type
- `current-version` - Version from Cargo.toml
- `new-version` - Next version number
- `new-tag` - Tag name (e.g., `v1.2.3`)

**Logic:**
1. Read version from `Cargo.toml`
2. Get commits since last tag
3. Check for breaking changes (`!` suffix)
4. Check for new features (`feat`)
5. Check for patch-worthy commits (`fix`, `perf`, etc.)
6. Calculate new version number
7. Validate tag doesn't already exist

**Usage:**
```yaml
- uses: ./.github/actions/version-bump
  id: version
  with:
    force_bump: ${{ github.event.inputs.force_bump }}

- name: Print new version
  run: echo "New version: ${{ steps.version.outputs.new-version }}"
```

### Update Changelog Action

**File:** `.github/actions/update-changelog.yml`

Updates `CHANGELOG.md` with new version section.

**Inputs:**
- `tag` - Tag name for the release

**Steps:**
1. Check if `CHANGELOG.md` exists
2. Create temporary file with new version header
3. Generate changelog with git-cliff for the tag
4. Append existing changelog (preserve history)
5. Replace `CHANGELOG.md` with updated content

**Usage:**
```yaml
- uses: ./.github/actions/update-changelog
  with:
    tag: ${{ steps.version.outputs.new-tag }}
```

---

## 6. Local Git Hooks

### Pre-push Hook

**File:** `.git/hooks/pre-push`

**Purpose:** Runs local checks before pushing to remote.

**Checks:**
1. **Formatting**: `cargo +nightly fmt` (auto-formats and fails if changes made)
2. **Clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests**: `cargo test --all-features`
4. **Security**: `cargo audit` (if installed)
5. **Outdated Dependencies**: `cargo outdated` (warning only, if installed)

**Installation:**

The pre-push hook is installed in `.git/hooks/pre-push` and is executable.

**Skipping Checks:**

To bypass pre-push checks (not recommended):
```bash
git push --no-verify
```

**Hook Behavior:**

```bash
🔧 Running pre-push checks...

📝 Checking rustfmt...
✅ rustfmt passed

🔍 Running clippy...
✅ clippy passed

🧪 Running tests...
✅ tests passed

🔒 Running cargo audit...
✅ cargo audit passed

📦 Checking outdated dependencies...
⚠️  2 outdated dependencies found. Run 'cargo outdated' for details.

🎉 All pre-push checks passed!
```

---

## 7. Configuration Files

### .cliff.toml

**Purpose:** git-cliff configuration for changelog generation.

**Key Sections:**
- `[changelog]` - Changelog header, body, and footer templates
- `[git]` - Commit parsers, filters, tag patterns

### CHANGELOG.md

**Purpose:** Automatically generated changelog.

**Structure:**
```markdown
# Changelog

## [v1.2.0] - 2026-01-26
<git-cliff generated content>

## [v1.1.0] - 2026-01-20
<git-cliff generated content>

## [v1.0.0] - 2026-01-15
<git-cliff generated content>
```

**Maintenance:**
- DO NOT manually edit sections
- New versions are prepended by CI/CD
- git-cliff generates content based on commits

---

## 8. Workflow Examples

### Example 1: New Feature Release

**Commits:**
```
feat(api): add transaction pagination
feat(ai): implement batch categorization
docs: update API documentation
chore(deps): update reqwest to 0.12
```

**Result:**
- Version bump: `v1.1.0` → `v1.2.0` (minor bump from `feat`)
- Changelog: Grouped by type (Features, Documentation, Dependencies)
- Release: GitHub release with changelog and binary

### Example 2: Bug Fix Release

**Commits:**
```
fix(api): handle pagination edge cases
fix(domain): correct transaction validation
test: add pagination unit tests
```

**Result:**
- Version bump: `v1.2.0` → `v1.2.1` (patch bump from `fix`)
- Changelog: Bug Fixes section
- Release: GitHub release with changelog and binary

### Example 3: Breaking Change Release

**Commits:**
```
feat!(api): remove deprecated transaction endpoints
fix!(domain): change transaction id format from int to UUID
docs: document breaking changes
```

**Result:**
- Version bump: `v1.2.1` → `v2.0.0` (major bump from `!`)
- Changelog: Breaking changes prominently displayed
- Release: GitHub release with migration notes

---

## 9. Troubleshooting

### Commits Workflow Fails

**Issue:** PR fails with "commits do not follow conventional commit format"

**Solution:**
```bash
# Interactive rebase to fix commits
git rebase -i origin/main

# Change 'pick' to 'reword' for invalid commits
# Update commit messages to follow format
```

**Example Fixes:**
- ❌ `add new feature` → ✅ `feat(api): add new feature`
- ❌ `fix bug` → ✅ `fix(api): fix transaction pagination bug`

### Release Workflow Fails

**Issue:** Release workflow fails with "Tag already exists"

**Solution:**
```bash
# Check existing tags
git tag -l

# Delete local tag (if duplicate)
git tag -d v1.2.3

# Delete remote tag (if needed)
git push origin :refs/tags/v1.2.3

# Retry release with force bump
# Go to GitHub Actions → Release workflow → Run workflow
# Select force_bump: patch/minor/major
```

### CI Test Failures

**Issue:** CI tests fail locally but pass on CI

**Solution:**
```bash
# Check CI environment variables
# CI uses: DATABASE_URL, KAFKA_BOOTSTRAP_SERVERS, etc.

# Run tests with same environment
export DATABASE_URL="postgresql://postgres:postgres@localhost:5433/postgres?schema=transaction_service"
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"
export OPENAI_API_KEY="sk-test-key"
export JWT_SECRET="test-secret"

cargo test --all-features
```

### Clippy Warnings

**Issue:** Pre-push hook fails with clippy warnings

**Solution:**
```bash
# Run clippy to see warnings
cargo clippy --all-targets --all-features

# Fix warnings automatically (some)
cargo clippy --all-targets --all-features --fix

# Manual fixes required for complex warnings
```

---

## 10. Best Practices

### Commit Messages

✅ **DO:**
- Use conventional commit format
- Add scope for clarity: `feat(api): ...`
- Use imperative mood: "add" not "added"
- Keep descriptions concise (50 chars or less)
- Mark breaking changes with `!`

❌ **DON'T:**
- Vague messages: "update stuff"
- Mixed changes: "feat and fix together"
- Skip scope on complex changes
- Forget `!` on breaking changes

### Release Management

✅ **DO:**
- Let CI/CD handle releases automatically
- Check CHANGELOG before release
- Test release binary locally
- Keep commits atomic and focused

❌ **DON'T:**
- Manually edit CHANGELOG.md
- Manually create tags
- Skip CI checks on main branch
- Force push to main

### Workflow Permissions

✅ **DO:**
- Use minimal required permissions
- Separate CI and release workflows
- Use secrets for sensitive data
- Validate workflow syntax before committing

❌ **DON'T:**
- Grant unnecessary permissions
- Hardcode secrets in workflows
- Skip security audits
- Ignore workflow failures

---

## 11. Summary

This repository implements a robust CI/CD pipeline with:

1. **Conventional commits** - Enforced structure for automated versioning
2. **Automated CI** - Tests, linting, builds, security on every push
3. **Semantic versioning** - Automatic version calculation based on commit types
4. **Automated releases** - GitHub releases with changelogs and binaries
5. **Local quality checks** - Pre-push hook for developer safety
6. **Changelog generation** - Automatic CHANGELOG.md updates with git-cliff

The workflows ensure code quality, maintain consistency, and automate repetitive tasks while providing visibility into changes through structured commit messages and generated changelogs.

---

[← Prerequisites](01-prerequisites.md) | [Next: Quick Start Checklist →](26-quick-start-checklist.md)
