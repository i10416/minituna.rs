use async_trait::async_trait;

use rand::{
    distributions::{uniform::SampleUniform, Uniform},
    prelude::Distribution,
    thread_rng,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    sampler::Sampler,
    storage::{InmemoryStorage, Storage},
};

pub struct Trial<T: Storage> {
    // reference to storage
    pub storage: T,
    pub trial_id: usize,
    pub study_id: Uuid,
}

impl Trial<InmemoryStorage> {
    pub(crate) fn new(study_id: Uuid, trial_id: usize) -> Self {
        Self {
            trial_id: trial_id,
            study_id: study_id,
            storage: InmemoryStorage::new(),
        }
    }
}



#[async_trait]
impl<T: Storage + Send + Sync> Sampler for Trial<T> {
    async fn sample_uniform<
        D: SampleUniform + Send + Sync + Copy + Serialize,
        V: PartialOrd + Send + Sync + Serialize,
    >(
        &self,
        name: &str,
        low: D,
        high: D,
    ) -> D {
        let param = Uniform::new(low, high).sample(&mut thread_rng());
        self.storage
            .set_trial_param::<D, V>(self.study_id.into(), self.trial_id, (name, param.clone()))
            .await;
        param
    }
}
