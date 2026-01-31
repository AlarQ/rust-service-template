# Rust Service CLI (rsc)

A CLI tool for creating and scaffolding Rust microservices from the service template. This tool automates the process of setting up new Rust services with Domain-Driven Design (DDD) architecture, complete with GitHub repository creation and optional Kafka support.

## Features

- **GitHub Integration**: Automatically create repositories and push initial commits
- **Local Scaffolding**: Generate services locally without GitHub integration
- **Kafka Support**: Optional Kafka event streaming support (can be excluded with `--without-kafka`)
- **DDD Architecture**: Generates services following Domain-Driven Design patterns
- **Pre-configured Stack**: Axum, SQLx, PostgreSQL, JWT authentication, OpenAPI docs

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-service-template.git
cd rust-service-template

# Build the CLI
cargo build --release --bin rsc

# Install to your PATH (optional)
cp target/release/rsc ~/.local/bin/
```

### Prerequisites

- Rust 1.70+ (stable)
- Git
- For `create` command: GitHub personal access token

## Configuration

### GitHub Token Setup

To use the `create` command, you need a GitHub personal access token:

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate a new token with `repo` scope (for private repos) or `public_repo` (for public repos)
3. Set the environment variable:

```bash
export GITHUB_TOKEN="your_token_here"
```

Add this to your shell profile (`.bashrc`, `.zshrc`, etc.) for persistence.

## Usage

### Create Command

Creates a new GitHub repository and generates a fully configured Rust microservice:

```bash
# Create a public repository with Kafka support
rsc create my-service --github-user myusername

# Create a private repository
rsc create my-service --github-user myusername --private

# Create without Kafka support
rsc create my-service --github-user myusername --without-kafka

# Create with description
rsc create my-service --github-user myusername --description "My awesome service"
```

### Scaffold Command

Generates a service locally without creating a GitHub repository:

```bash
# Scaffold in current directory
rsc scaffold my-service

# Scaffold to specific directory
rsc scaffold my-service --output /path/to/output

# Scaffold without Kafka support
rsc scaffold my-service --without-kafka
```

## CLI Reference

### Global Options

The `rsc` CLI supports the following commands:

#### `create`

Create a new repository on GitHub with generated service files.

```
rsc create <NAME> --github-user <USER> [OPTIONS]
```

**Arguments:**
- `NAME` - Name of the repository/service to create

**Options:**
- `-u, --github-user <USER>` - GitHub username or organization (required)
- `-p, --private` - Create a private repository (default: public)
- `-d, --description <DESC>` - Description for the repository
- `--without-kafka` - Exclude Kafka support from the generated service

#### `scaffold`

Scaffold a new service locally without creating a GitHub repository.

```
rsc scaffold <NAME> [OPTIONS]
```

**Arguments:**
- `NAME` - Name of the service to scaffold

**Options:**
- `-o, --output <PATH>` - Output directory for the scaffolded service (default: `./<NAME>`)
- `--without-kafka` - Exclude Kafka support from the generated service

## Generated Service Structure

The generated service follows Domain-Driven Design principles:

```
my-service/
├── src/
│   ├── domain/           # Core business logic
│   ├── infrastructure/   # External integrations
│   ├── api/             # HTTP layer (Axum handlers)
│   └── main.rs          # Application entry point
├── tests/               # Integration tests
├── migrations/          # SQLx database migrations
├── docker-compose.yaml  # Development dependencies
└── run.sh              # Development startup script
```

### Features Included

- **Axum** web framework with middleware support
- **SQLx** for type-safe database queries (PostgreSQL)
- **JWT** authentication with claims extraction
- **OpenAPI** documentation via utoipa
- **Tracing** for structured logging
- **Kafka** event streaming (optional)
- **Health checks** (liveness and readiness)
- **CORS** configuration
- **Git hooks** for code quality

## Development

### Running the CLI locally

```bash
# Set your GitHub token
export GITHUB_TOKEN="your_token"

# Run with cargo
cargo run --bin rsc -- create my-service --github-user myusername
```

### Running Tests

```bash
# Run all tests
cargo test

# Run only CLI tests
cargo test --bin rsc
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For issues and feature requests, please use the GitHub issue tracker.
