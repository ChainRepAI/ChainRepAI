use pulsar::{Pulsar, TokioExecutor};

pub struct PulsarClient {
    internal_client: Pulsar<TokioExecutor>,
}
