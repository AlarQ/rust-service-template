# Code Quality Configuration

[← Prerequisites](01-prerequisites.md) | [Next: Project Structure →](03-project-structure.md)

---

## Clippy Configuration

Add to the top of `src/main.rs`:

```rust
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
```

---

## Rustfmt Configuration

Create `rustfmt.toml` in project root:

```toml
imports_granularity = "Crate"
brace_style = "SameLineWhere"
```

---

[← Prerequisites](01-prerequisites.md) | [Next: Project Structure →](03-project-structure.md)
