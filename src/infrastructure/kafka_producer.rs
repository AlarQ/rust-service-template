use std::time::Duration;

use async_trait::async_trait;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use tracing::{debug, error, info};

use crate::{
    config::KafkaConfig,
    domain::{
        errors::DomainError, interfaces::event_producer::EventProducer,
        task::models::events::TaskEvent,
    },
};

/// Kafka event service for publishing task events
pub struct KafkaEventService {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventService {
    /// Create a new Kafka event service with the given configuration
    ///
    /// # Errors
    /// Returns `DomainError::ExternalError` if the producer cannot be created
    pub fn new(config: &KafkaConfig) -> Result<Self, DomainError> {
        info!(
            "Initializing Kafka producer with bootstrap servers: {}",
            config.bootstrap_servers
        );

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("client.id", &config.client_id)
            .set("message.timeout.ms", "10000")
            .set("acks", "all")
            .set("retries", "3")
            .set("retry.backoff.ms", "1000")
            .create()
            .map_err(|e| {
                DomainError::external_error(format!("Failed to create Kafka producer: {e}"))
            })?;

        info!(
            "Kafka producer initialized successfully for topic: {}",
            config.task_topic
        );

        Ok(Self {
            producer,
            topic: config.task_topic.clone(),
        })
    }
}

#[async_trait]
impl EventProducer for KafkaEventService {
    async fn publish_task_event(&self, event: TaskEvent) -> Result<(), DomainError> {
        let event_json = serde_json::to_string(&event).map_err(|e| {
            DomainError::external_error(format!("Failed to serialize task event: {e}"))
        })?;

        let event_id = event.event_id.to_string();
        let task_id = event.data.id.to_string();

        debug!(
            "Publishing task event to Kafka: event_id={}, event_type={:?}, topic={}",
            event_id, event.event_type, self.topic
        );

        let record = FutureRecord::to(&self.topic)
            .key(&task_id)
            .payload(&event_json)
            .headers(
                rdkafka::message::OwnedHeaders::new().insert(rdkafka::message::Header {
                    key: "event_type",
                    value: Some(&format!("{:?}", event.event_type)),
                }),
            );

        let timeout = Duration::from_secs(10);

        match self.producer.send(record, timeout).await {
            Ok(delivery) => {
                info!(
                    "Successfully published task event to Kafka: event_id={}, partition={}, offset={}",
                    event_id, delivery.partition, delivery.offset
                );
                Ok(())
            }
            Err((e, _)) => {
                error!(
                    "Failed to publish task event to Kafka: event_id={}, error={}",
                    event_id, e
                );
                Err(DomainError::external_error(format!(
                    "Failed to publish event to Kafka: {e}"
                )))
            }
        }
    }
}
