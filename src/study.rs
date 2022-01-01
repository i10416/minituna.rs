use std::cmp::Ordering;

use crate::{
    frozen_trial::TrialData,
    storage::{InmemoryStorage, Storage},
    trial::Trial,
};
use async_trait::async_trait;
use futures::Future;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;
use uuid::Uuid;
#[async_trait]
trait Study<T: Storage + 'static, R: PartialOrd> {
    async fn get_best_trial(&self) -> Option<TrialData<R>>;

    async fn optimize<F, G>(&mut self, f: F, n_trial: usize)
    where
        F: Sync + Send + Fn(Trial<T>) -> G,
        G: Future<Output = R> + Send + 'static;
}
struct StudyInstance<T: Storage> {
    id: Uuid,
    storage: T,
}

#[async_trait]
impl<'a, V: PartialOrd + Send + Sync + Copy + Serialize> Study<InmemoryStorage, V>
    for StudyInstance<InmemoryStorage>
{
    async fn get_best_trial(&self) -> Option<TrialData<V>> {
        let it = self.storage.get_trials::<V>(self.id.into()).await;
        let that = it.into_iter().max_by(|x, y| match (x, y) {
            (
                TrialData::Complete {
                    trial_value: trial_value_x @ _,
                    ..
                },
                TrialData::Complete {
                    trial_value: trial_value_y @ _,
                    ..
                },
            ) => trial_value_x.partial_cmp(trial_value_y).unwrap(),
            (TrialData::Complete { .. }, TrialData::InProgress { .. }) => Ordering::Greater,
            (TrialData::InProgress { .. }, TrialData::Complete { .. }) => Ordering::Less,
            (TrialData::InProgress { .. }, TrialData::InProgress { .. }) => Ordering::Equal,
        });
        match that {
            best_one @ Some(TrialData::Complete { .. }) => best_one,
            _ => None,
        }
    }
    async fn optimize<F, G>(&mut self, f: F, n_trial: usize)
    where
        F: Sync + Send + Fn(Trial<InmemoryStorage>) -> G,
        G: Future<Output = V> + Send + 'static,
    {
        let ts: Vec<Trial<InmemoryStorage>> = (0..n_trial)
            .into_par_iter()
            .map(|n| Trial::new(self.id, n))
            //storage.create_trial のように storage が trial を生成するようにすれば、InmemoryStorage ではなく T: Storage +'static に抽象化できるか.
            .collect();
        // S => E[F[T]]
        let fs = ts.into_iter().map(|t| async {
            let i = t.trial_id;
            let res = f(t).await;
            (i, res)
        });
        // traverse
        let res: Vec<(usize, V)> = futures::future::join_all(fs).await;
        // T => G[F[U]]
        let (_, futs) = res.into_iter().fold(
            (&self.storage, Vec::new()),
            |(storage, mut futs), (u, v)| {
                futs.push(storage.set_trial_value(self.id.into(), u, v));
                (storage, futs)
            },
        );
        // traverse
        futures::future::join_all(futs.into_iter()).await;
    }
}

mod test {

    use uuid::Uuid;

    use crate::{sampler::Sampler, storage::InmemoryStorage};

    use super::{Study, StudyInstance};

    async fn test() {
        let mut s = StudyInstance {
            id: Uuid::new_v4(),
            storage: InmemoryStorage::new(),
        };
        s.optimize(
            |t| async move {
                let x = t.sample_uniform::<i32, f32>("x", 0, 100).await;
                let y = t.sample_uniform::<f32, f32>("y", -100.0, 100.0).await;
                (x * x) as f32 + y
            },
            100,
        )
        .await;
    }
}
