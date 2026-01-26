# Value Objects Pattern

[← Domain Services](16-domain-services.md) | [Next: DTO Conversion →](18-dto-conversion.md)

---

Value objects encapsulate validation and ensure domain invariants.

## Newtype Pattern with Validation

```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Amount value object with validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Amount(Decimal);

impl Amount {
    pub fn new(value: Decimal) -> Result<Self, ValidationError> {
        // Business rule: amount cannot be zero
        if value.is_zero() {
            return Err(ValidationError::InvalidAmount("Amount cannot be zero".into()));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }

    pub fn is_expense(&self) -> bool {
        self.0.is_sign_negative()
    }

    pub fn is_income(&self) -> bool {
        self.0.is_sign_positive()
    }
}

/// Description value object with length validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Description(String);

impl Description {
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: usize = 500;

    pub fn new(value: String) -> Result<Self, ValidationError> {
        let trimmed = value.trim().to_string();

        if trimmed.len() < Self::MIN_LENGTH {
            return Err(ValidationError::InvalidDescription(
                "Description cannot be empty".into(),
            ));
        }

        if trimmed.len() > Self::MAX_LENGTH {
            return Err(ValidationError::InvalidDescription(format!(
                "Description cannot exceed {} characters",
                Self::MAX_LENGTH
            )));
        }

        Ok(Self(trimmed))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid description: {0}")]
    InvalidDescription(String),

    #[error("Invalid {field}: {message}")]
    InvalidField { field: String, message: String },
}
```

---

## Entity ID Newtype Wrapper

```rust
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct {Feature}Id(pub Uuid);

impl {Feature}Id {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for {Feature}Id {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for {Feature}Id {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for {Feature}Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

---

## Entity with Validated Construction

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct {Feature} {
    pub id: {Feature}Id,
    pub user_id: UserId,
    pub amount: Amount,
    pub description: Description,
    pub category_id: CategoryId,
    pub created_at: DateTime<Utc>,
}

impl {Feature} {
    /// Create a new entity with validation
    pub fn new(
        user_id: UserId,
        amount: Decimal,
        description: String,
        category_id: CategoryId,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id: {Feature}Id::new(),
            user_id,
            amount: Amount::new(amount)?,
            description: Description::new(description)?,
            category_id,
            created_at: Utc::now(),
        })
    }
}
```

---

## Benefits

| Pattern | Benefit |
|---------|---------|
| Newtype wrappers | Type safety, prevents mixing IDs |
| Validation in constructor | Invalid states unrepresentable |
| `From` impls | Easy conversion from primitives |
| `Display` impl | Clean logging and error messages |

---

[← Domain Services](16-domain-services.md) | [Next: DTO Conversion →](18-dto-conversion.md)
