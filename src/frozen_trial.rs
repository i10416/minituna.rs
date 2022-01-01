use std::{collections::HashMap};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum TrialData<V> {
    Complete {
        #[serde(with = "StudyId")]
        study_id: Uuid,
        trial_id: usize,
        trial_param: HashMap<String, String>,
        trial_value: V,
    },
    InProgress {
        #[serde(with = "StudyId")]
        study_id: Uuid,
        trial_id: usize,
        // trial_params: HashMap<String,String>
        trial_params: HashMap<String, String>,
    },
    // Pruned {}
}

#[derive(Serialize, Deserialize)] // implicitly require T to be deserializable
pub struct ParamData<T> {
    value: T,
    dist_name: String,
}

// should be T:Unique+Hash+PartialEq+Eq instead of Uuid?

#[derive(Serialize, Deserialize)]
#[serde(remote = "Uuid")]
struct StudyId {
    #[serde(getter = "Uuid::to_string")]
    value: String,
}
impl From<StudyId> for Uuid {
    fn from(study_id: StudyId) -> Uuid {
        Uuid::parse_str(&study_id.value).unwrap()
    }
}
