use async_trait::async_trait;
use rand::distributions::uniform::SampleUniform;
use serde::Serialize;

#[async_trait]
pub trait Sampler {
    async fn sample_uniform<
        D: SampleUniform + Send + Sync + Copy + Serialize,
        V: PartialOrd + Send + Sync + Serialize,
    >(
        &self,
        name:&str,
        low: D,
        high: D,
    ) -> D;
}

