//! Interface to the sampler
use nuts_rs::{new_sampler, Chain, CpuLogpFunc, SampleStats, SamplerArgs};

use rand::rngs::SmallRng;
use rand::SeedableRng;

#[derive(Debug)]
pub struct MyDivergenceInfo {
    pub start_momentum: Option<Box<[f64]>>,
    pub start_location: Option<Box<[f64]>>,
    pub start_gradient: Option<Box<[f64]>>,
    pub end_location: Option<Box<[f64]>>,
    pub energy_error: Option<f64>,
    pub end_idx_in_trajectory: Option<i64>,
    pub start_idx_in_trajectory: Option<i64>,
}

impl From<&nuts_rs::DivergenceInfo> for MyDivergenceInfo {
    fn from(div_info: &nuts_rs::DivergenceInfo) -> Self {
        Self {
            start_momentum: div_info.start_momentum.clone(),
            start_location: div_info.start_location.clone(),
            start_gradient: div_info.start_gradient.clone(),
            end_location: div_info.end_location.clone(),
            energy_error: div_info.energy_error,
            end_idx_in_trajectory: div_info.end_idx_in_trajectory,
            start_idx_in_trajectory: div_info.start_idx_in_trajectory,
        }
    }
}

/// Run the sampler
pub fn be_nuts<F>(
    logp_func: F,
    num_tune: u64,
    num_samples: u64,
    seed: u64,
) -> (Vec<Box<[f64]>>, Vec<MyDivergenceInfo>)
where
    F: CpuLogpFunc,
{
    // We get the default sampler arguments
    let mut sampler_args = SamplerArgs::default();

    let dim = logp_func.dim();
    sampler_args.num_tune = num_tune;

    let chain = 0;
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut sampler = new_sampler(logp_func, sampler_args, chain, &mut rng);

    // Set to some initial position
    sampler
        .set_position(&vec![0f64; dim])
        .expect("Unrecoverable error during init");

    // Burn the first x samples to get away from the initial position
    for _ in 0..50 {
        sampler.draw().expect("Unrecoverable error during burning");
    }

    let mut trace = vec![]; // Collection of all draws
    let mut stats = vec![]; // Collection of statistics like the acceptance rate for each draw
    for _ in 0..num_samples {
        let (draw, info) = sampler.draw().expect("Unrecoverable error during sampling");
        trace.push(draw);
        if let Some(div_info) = info.divergence_info() {
            stats.push(div_info.into());
        }
    }

    (trace, stats)
}
