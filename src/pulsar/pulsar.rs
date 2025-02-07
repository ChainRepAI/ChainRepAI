use anyhow::Result;
use log::{error, info};
use pulsar::{
    consumer::{DeadLetterPolicy, Message},
    producer, proto, Consumer, Producer, Pulsar, SubType, TokioExecutor,
};
use uuid::Uuid; // Import logging macros

use crate::jobs::async_jobs::WalletReportJob;

const PULSAR_ADDR: &str = "pulsar://localhost:6650";
const MAX_JOB_RETRY: usize = 3;

pub struct PulsarClient {
    internal_client: Pulsar<TokioExecutor>,
}

impl PulsarClient {
    pub async fn new() -> Self {
        // Create a new Pulsar client instance
        let client = Pulsar::builder(PULSAR_ADDR, TokioExecutor)
            .build()
            .await
            .expect("Should be able to create new pulsar client builder");
        info!(
            "Pulsar client created successfully with address: {}",
            PULSAR_ADDR
        );
        Self {
            internal_client: client,
        }
    }

    pub async fn create_producer(&self, topic: &str) -> PulsarProducer {
        let id = Uuid::new_v4();

        PulsarProducer {
            id,
            internal_producer: self
                .internal_client
                .producer()
                .with_topic(topic)
                .with_name("PRODUCER_".to_owned() + &id.to_string())
                .with_options(producer::ProducerOptions {
                    schema: Some(proto::Schema {
                        r#type: proto::schema::Type::String as i32,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build()
                .await
                .expect("Should be able to create producer"),
        }
    }

    pub async fn create_consumer(
        &self,
        topics: Vec<&str>,
        subscription_type: SubType,
        subscription: &str,
    ) -> PulsarConsumer {
        let id = Uuid::new_v4();
        PulsarConsumer {
            id,
            internal_consumer: self
                .internal_client
                .consumer()
                // .with_topic(topic)
                .with_consumer_name("CONSUMER_".to_owned() + &id.to_string())
                .with_subscription_type(subscription_type) // exclusive for current testing
                // .with_subscription("SUB_".to_owned() + &id.to_string())
                .with_subscription(subscription)
                .with_topics(topics)
                .with_dead_letter_policy(DeadLetterPolicy {
                    max_redeliver_count: MAX_JOB_RETRY,
                    dead_letter_topic: "DLQ".to_string(),
                })
                .build()
                .await
                .expect("Should be able to create consumer"),
        }
    }
}

pub struct PulsarProducer {
    id: Uuid,
    internal_producer: Producer<TokioExecutor>,
}

impl PulsarProducer {
    pub async fn enqueue_job(&mut self, event: WalletReportJob) -> Result<()> {
        self.internal_producer.send_non_blocking(event).await?;
        info!("Enqueuing job on producer with id: {}", self.id);
        Ok(())
    }
}

pub struct PulsarConsumer {
    id: Uuid,
    pub internal_consumer: Consumer<WalletReportJob, TokioExecutor>,
}

impl PulsarConsumer {
    pub async fn ack(&mut self, msg: &Message<WalletReportJob>) {
        info!(
            "Acknowledging message with id: {:?} on consumer: {}",
            msg.message_id(),
            self.id
        );
        if let Err(e) = self.internal_consumer.ack(msg).await {
            error!(
                "Failed to ack message with id: {:?} on consumer {}: {:?}",
                msg.message_id(),
                self.id,
                e
            );
        } else {
            info!(
                "Message with id: {:?} acknowledged successfully",
                msg.message_id()
            );
        }
    }

    pub async fn nack(&mut self, msg: &Message<WalletReportJob>) {
        info!(
            "Nacking message with id: {:?} on consumer: {}",
            msg.message_id(),
            self.id
        );
        if let Err(e) = self.internal_consumer.nack(msg).await {
            error!(
                "Failed to nack message with id: {:?} on consumer {}: {:?}",
                msg.message_id(),
                self.id,
                e
            );
        } else {
            info!(
                "Message with id: {:?} nacked successfully",
                msg.message_id()
            );
        }
    }
}
