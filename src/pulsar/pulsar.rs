use pulsar::{Producer, Pulsar, TokioExecutor};
use uuid::Uuid;

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
}

pub struct PulsarProducer {
    id: Uuid,
    internal_producer: Producer<TokioExecutor>,
}
