## minituna.rs

Simplified version of __Optuna__, famous Python library for hyper parameter tuning, in Rust.

API design is mainly based on [minituna](https://github.com/CyberAgentAILab/minituna) by [CyberAgentAILab](https://github.com/CyberAgentAILab).

## example

```rust
let mut s = StudyInstance::new();

s.optimize(
    |t| async move {
        let x = t.sample_uniform::<i32, f32>("x", 0, 100).await;
        let y = t.sample_uniform::<f32, f32>("y", -100.0, 100.0).await;
        (x * x) as f32 + y
    },
    100,
)
.await;
s.get_best_trial().await;
```
## difference from minituna

`minituna.rs`

- has async APIs. IO operations won't block(,just semantically block.)(as long as developers handle them appropriately.) Independent trials can run in parallel. 

- has generic samplers.

For example, `sample_uniform` function accepts every type which satisfies these type class instance bounds.

```rust
async fn sample_uniform<
        D: SampleUniform + Send + Sync + Copy + Serialize,
        V: PartialOrd + Send + Sync + Serialize,
    >(
        &self,
        name:&str,
        low: D,
        high: D,
    ) -> D
``` 

You can use sample method with type parameters.
In this example below, x is `i32` while `y` is `f32`.

```rust
        s.optimize(
            |t| async move {
                let x = t.sample_uniform::<i32, f32>("x", 0, 100).await;
                let y = t.sample_uniform::<f32, f32>("y", -100.0, 100.0).await;
                (x * x) as f32 + y
            },
            100,
        )
        .await;
```

- can save multiple studies in storage.

For instance, storaga has these APIs for querying and updating trial data for a specific study. In addition, as write APIs require just `Serialize`, you can implement and use different storage layers for different serialization formats.

```rust
    async fn get_trials<V: PartialOrd + Send + Sync>(&self, study_id: Uuid)
        -> Vec<TrialData<V>>;
    async fn set_trial_value<V: PartialOrd + Send + Sync + Serialize>(
        &self,
        study_id: Uuid,
        trial_id: usize,
        value: V,
    );
```

- async runtime agnostic. designed to `futures` crate api.
