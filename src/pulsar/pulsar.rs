use pulsar::{consumer::Message, producer, proto, Consumer, Producer, Pulsar, TokioExecutor};
use uuid::Uuid;

use crate::jobs::jobs::WalletReportJob;

const PULSAR_ADDR: &str = "pulsar://localhost:6650";

pub struct PulsarClient {
    internal_client: Pulsar<TokioExecutor>,
}

impl PulsarClient {
    pub async fn new() -> Self {
        Self {
            internal_client: Pulsar::builder(PULSAR_ADDR, TokioExecutor)
                .build()
                .await
                .expect("Should be able to create new pulsar client builder"),
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
}

pub struct PulsarProducer {
    id: Uuid,
    internal_producer: Producer<TokioExecutor>,
}

pub struct PulsarConsumer {
    id: Uuid,
    pub internal_consumer: Consumer<WalletReportJob, TokioExecutor>,
}

impl PulsarConsumer {
    pub async fn ack(&mut self, msg: &Message<WalletReportJob>) {
        if let Err(_) = self.internal_consumer.ack(msg).await {
            println!("Failed to ack message with id: {:?}", msg.message_id());
        }
    }

    pub async fn nack(&mut self, msg: &Message<WalletReportJob>) {
        if let Err(_) = self.internal_consumer.nack(msg).await {
            println!("Failed to nack message with id: {:?}", msg.message_id());
        }
    }
}
