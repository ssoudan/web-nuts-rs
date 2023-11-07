//! The model for the example is a 2D normal distribution with mean 3.
use std::fmt::Display;
use std::fmt::Formatter;

use nuts_rs::{CpuLogpFunc, LogpError};

#[derive(Debug, Clone)]
pub(crate) struct Model {
    pub(crate) observed: Vec<Vec<f64>>,
    pub(crate) dims: usize,
    pub(crate) parameters: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum ModelError {}

impl std::error::Error for ModelError {}
impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Some error")
    }
}

impl LogpError for ModelError {
    fn is_recoverable(&self) -> bool {
        true
    }
}

const SIGMA: f64 = 1.;

impl CpuLogpFunc for Model {
    type Err = ModelError;

    fn dim(&self) -> usize {
        self.dims
    }

    fn logp(&mut self, position: &[f64], grad: &mut [f64]) -> Result<f64, Self::Err> {
        assert_eq!(position.len(), self.dims);

        // position = [mu1, mu2]
        // observed = [[obs1_1, obs1_2], [obs2_1, obs2_2], ...]

        // logp \propto - sum_i ||obs_i - mu||^2 / 2 / sigma^2
        // grad[j] = sum_i (obs_i[j] - mu[j]) / sigma^2

        for d in 0..self.dims {
            grad[d] = self
                .observed
                .iter()
                .map(|obs| obs[d] - position[d])
                .sum::<f64>()
                / SIGMA.powi(2);
        }

        let logp = self
            .observed
            .iter()
            .map(|obs| {
                let norm_sq = obs
                    .iter()
                    .zip(position)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>();
                -norm_sq / 2f64 / SIGMA.powi(2)
            })
            .sum::<f64>();

        Ok(logp)
    }
}
