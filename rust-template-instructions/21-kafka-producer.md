# Kafka Producer Configuration

[← Event Schema](20-event-schema.md) | [Next: Testing →](22-testing.md)

---

## Producer Implementation

```rust
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::time::Duration;

use crate::{
    config::KafkaConfig,
    domain::{
        interfaces::event_producer::EventProducer,
        {feature}::models::{Feature}Event,
    },
};

#[derive(Debug)]
pub struct KafkaEventService {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventService {
    pub fn new(config: &KafkaConfig) -> Result<Self, anyhow::Error> {
        let producer: FutureProducer = ClientConfig::new()
            // Connection
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("client.id", &config.client_id)
            // Reliability settings
            .set("acks", "all")                        // Wait for all replicas
            .set("retries", "3")                       // Retry failed sends
            .set("enable.idempotence", "true")         // Prevent duplicates
            .set("max.in.flight.requests.per.connection", "1")  // Ordering guarantee
            // Performance tuning
            .set("batch.size", "16384")                // 16KB batch
            .set("linger.ms", "5")                     // Wait up to 5ms for batching
            .set("compression.type", "lz4")            // Enable compression
            // Timeouts
            .set("request.timeout.ms", "30000")
            .set("delivery.timeout.ms", "120000")
            .create()
            .map_err(|e| anyhow::anyhow!("Failed to create Kafka producer: {e}"))?;

        Ok(Self {
            producer,
            topic: config.{feature}_topic.clone(),
        })
    }
}

#[async_trait::async_trait]
impl EventProducer for KafkaEventService {
    async fn publish_{feature}_event(
        &self,
        event: {Feature}Event,
    ) -> Result<(), {Feature}Error> {
        let event_json = serde_json::to_string(&event)
            .map_err(|e| {Feature}Error::external_error(format!("Serialization error: {e}")))?;

        // Use entity ID as partition key for ordering per entity
        let key = event.data.id.to_string();

        let record = FutureRecord::to(&self.topic)
            .key(&key)
            .payload(&event_json);

        match self
            .producer
            .send(record, Duration::from_secs(10))
            .await
        {
            Ok((partition, offset)) => {
                tracing::info!(
                    event_id = %event.event_id,
                    event_type = ?event.event_type,
                    partition = partition,
                    offset = offset,
                    "Published event to Kafka"
                );
                Ok(())
            }
            Err((e, _)) => {
                tracing::error!(
                    event_id = %event.event_id,
                    error = %e,
                    "Failed to publish event to Kafka"
                );
                Err({Feature}Error::external_error(format!("Kafka publish error: {e}")))
            }
        }
    }
}
```

---

## Event Producer Interface

```rust
// domain/interfaces/event_producer.rs
use async_trait::async_trait;

use crate::domain::{feature}::models::{Feature}Event;

#[async_trait]
pub trait EventProducer: Send + Sync + std::fmt::Debug {
    async fn publish_{feature}_event(
        &self,
        event: {Feature}Event,
    ) -> Result<(), {Feature}Error>;
}
```

---

## Configuration

```rust
// In config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub {feature}_topic: String,
    pub client_id: String,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            {feature}_topic: "int.{feature}-update.event".to_string(),
            client_id: "{service-name}".to_string(),
        }
    }
}
```

---

## Non-Blocking Event Publishing

Events should not block the main operation:

```rust
// In domain services
pub async fn create_{feature}(
    entity: {Feature},
    repo: Arc<dyn {Feature}Repository>,
    events: Arc<dyn EventProducer>,
    category_repo: Arc<dyn CategoryRepository>,
) -> Result<{Feature}, {Feature}Error> {
    // Create entity in database
    let created = repo.create(entity).await?;

    // Get category name for event
    let category = category_repo
        .get(created.category_id.clone())
        .await?
        .ok_or_else(|| {Feature}Error::not_found("Category", created.category_id.to_string()))?;

    // Publish event - log error but don't fail the operation
    let correlation_id = uuid::Uuid::new_v4();
    let event = {Feature}Event::new_created(&created, category.name, correlation_id);

    if let Err(e) = events.publish_{feature}_event(event).await {
        tracing::error!(
            error = %e,
            {feature}_id = %created.id,
            "Failed to publish event - operation succeeded but event not published"
        );
        // Consider: Add to outbox table for retry
    }

    Ok(created)
}
```

---

## Producer Settings Reference

| Setting | Value | Purpose |
|---------|-------|---------|
| `acks` | `all` | Wait for all replicas |
| `enable.idempotence` | `true` | Prevent duplicates |
| `compression.type` | `lz4` | Reduce bandwidth |
| `batch.size` | `16384` | 16KB batching |
| `linger.ms` | `5` | Wait for batch fill |

---

[← Event Schema](20-event-schema.md) | [Next: Testing →](22-testing.md)
