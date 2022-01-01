use std::collections::HashMap;

use crate::frozen_trial::{TrialData};
use async_trait::async_trait;
use futures::lock::Mutex;
use once_cell::sync::OnceCell;
use serde::{Serialize};
use serde_json;
use uuid::Uuid;
#[async_trait]
pub trait Storage {
    // fetch from trial data source and frozen trial data source
    async fn get_trial<V: PartialOrd + Send + Sync>(
        &self,
        study_id: Uuid,
        trial_id: usize,
    ) -> Option<TrialData<V>>; // [FrozenTrial]
    async fn get_trials<V: PartialOrd + Send + Sync>(&self, study_id: Uuid)
        -> Vec<TrialData<V>>; // [FrozenTrial]
                              // fetch from frozen trial datasource
    async fn set_trial_value<V: PartialOrd + Send + Sync + Serialize>(
        &self,
        study_id: Uuid,
        trial_id: usize,
        value: V,
    );
    async fn set_trial_param<T: Send + Sync + Serialize, V: PartialOrd + Send + Sync + Serialize>(
        &self,
        study_id: Uuid,
        trial_id: usize,
        param: (&str, T),
    );
}

pub struct InmemoryStorage {
    underlying: &'static Mutex<HashMap<Uuid, Box<HashMap<usize, String>>>>,
}

impl InmemoryStorage {
    fn instance() -> &'static Mutex<HashMap<Uuid, Box<HashMap<usize, String>>>> {
        static INSTANCE: OnceCell<Mutex<HashMap<Uuid, Box<HashMap<usize, String>>>>> =
            OnceCell::new();
        INSTANCE.get_or_init(|| Mutex::new(HashMap::new()))
    }
    pub fn new() -> Self {
        Self {
            underlying: InmemoryStorage::instance(),
        }
    }
}

#[async_trait]

impl Storage for InmemoryStorage {
    async fn get_trial<V: PartialOrd + Send + Sync>(
        &self,
        study_id: Uuid,
        trial_id: usize,
    ) -> Option<TrialData<V>> {
        let it = &mut *self.underlying.lock().await;
        if let Some(hm) = it.get(&study_id) {
            if let Some(trial_data) = hm.get(&trial_id) {
                
            // let decode_result:Result<TrialData<V>,serde_json::Error> = serde_json::from_str(trial_data);
            // decode_result.to_option
                todo!()
            }
        }
        None
    }

    // Vec[TrialData]
    async fn get_trials<V: PartialOrd + Send + Sync>(
        &self,
        study_id: Uuid,
    ) -> Vec<TrialData<V>> {
        let it = &mut *self.underlying.lock().await;
        if let Some(hm) = it.get(&study_id) {
            let trials = hm.values();
            // let decode_result:Result<Vec<TrialData<V>>,serde_json::Error> = serde_json::from_str(trials.map(_.1));
            // decode_result.get_or_else(Vec::new())
            todo!()
        }
        Vec::new()
    }

    async fn set_trial_value<V: PartialOrd + Send + Sync + Serialize>(
        &self,
        study_id: Uuid,
        trial_id: usize,
        value: V,
    ) {
        let maybe_ongoing = self.get_trial::<V>(study_id, trial_id).await;
        match maybe_ongoing {
            Some(TrialData::Complete { .. }) => panic!("Cannot set value to finished trial."),
            None => (), // panic?
            Some(TrialData::InProgress {
                study_id,
                trial_id,
                trial_params,
            }) => {
                let data = TrialData::Complete {
                    study_id: study_id,
                    trial_id: trial_id,
                    trial_param: trial_params,
                    trial_value: value,
                };
                let it = &mut *self.underlying.lock().await;
                let s = serde_json::to_string(&data).unwrap();
                // it(study_id)(trial_id)(s);
            }
        }
    }

    async fn set_trial_param<
        T: Send + Sync + Serialize,
        V: PartialOrd + Send + Sync + Serialize,
    >(
        &self,
        study_id: Uuid,
        trial_id: usize,
        param: (&str, T), // or ParamData<T>?
    ) {
        let maybe_ongoing = self.get_trial::<V>(study_id.clone(), trial_id).await;
        let data = match maybe_ongoing {
            Some(TrialData::Complete { .. }) => panic!("Cannot set params to finished trial."),
            None => TrialData::<V>::InProgress {
                study_id,
                trial_id: trial_id,
                trial_params: HashMap::<String, String>::from([(
                    String::from(param.0),
                    serde_json::to_string(&param.1).unwrap(),
                )]),
            }, //new
            Some(TrialData::InProgress {
                study_id,
                trial_id,
                trial_params,
            }) => {
                TrialData::<V>::InProgress {
                    study_id: study_id,
                    trial_id: trial_id,
                    trial_params: trial_params, // <- update this
                                                // trial_params.updated(String::from(params.0),serde_json::to_string(params.1)))
                }
            }
        };
        let it = &mut *self.underlying.lock().await;
        let s = serde_json::to_string(&data).unwrap();
        // it(study_id)(trial_id)(s);
    }
}
