//! # regression
use nuts_rs::{CpuLogpFunc, LogpError};

use crate::chain::Model;

/// A simple error type.
#[derive(Debug)]
pub(crate) enum RegressionError {
    /// Sigma is negative.
    NegativeSigma,
}

impl std::fmt::Display for RegressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RegressionError::NegativeSigma => write!(f, "Sigma is negative"),
        }
    }
}

impl std::error::Error for RegressionError {}

impl LogpError for RegressionError {
    fn is_recoverable(&self) -> bool {
        true
    }
}

/// A regression model.
#[derive(Clone)]
pub(crate) struct Regression {
    x: Vec<f64>,
    y: Vec<f64>,
}

impl Regression {
    /// Create a new regression model.
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        assert_eq!(x.len(), y.len(), "Dimension mismatch");
        Self { x, y }
    }
}

fn log_pdf_normal(x: f64, mu: f64, sigma: f64) -> f64 {
    let a = -0.5 * (2.0 * std::f64::consts::PI * sigma.powi(2)).ln();
    let b = -0.5 * ((x - mu) / sigma).powi(2);
    a + b
}

impl CpuLogpFunc for Regression {
    type Err = RegressionError;

    fn logp(&mut self, position: &[f64], grad: &mut [f64]) -> Result<f64, Self::Err> {
        // positions = alpha, beta, sigma

        // alpha: intercept
        // beta: slope
        // sigma: noise

        // We have to return the unnormalized log density of the distribution we want to
        // sample from. Since we are performing a Bayesian regression, we want
        // to sample from the posterior distribution. The posterior distribution
        // is proportional to the likelihood times the prior. Because we only need
        // the unnormalized log density, we can ignore the evidence term (the
        // denominator of Bayes' rule) as it is constant. Now since we are
        // dealing with the log density, we can simply add the log likelihood and the
        // log priors. The likelihood is a normal distribution with mean alpha +
        // beta * x and standard deviation sigma. And finally: we are using a
        // flat prior for sigma, so we can ignore it.

        // For the gradient, we need to compute the partial derivatives of the log
        // density with respect to the parameters: alpha, beta, sigma.

        const ALPHA: usize = 0;
        const BETA: usize = 1;
        const SIGMA: usize = 2;

        if position[SIGMA] <= 0.0 {
            return Err(RegressionError::NegativeSigma);
        }

        let logp_alpha = log_pdf_normal(position[ALPHA], 0.0, 10.0);
        let logp_beta = log_pdf_normal(position[BETA], 0.0, 10.0);
        let logp_sigma = 0.; // flat prior

        let mu = self
            .x
            .iter()
            .map(|x| position[ALPHA] + position[BETA] * x)
            .collect::<Vec<_>>();
        let logp_y = self
            .y
            .iter()
            .zip(mu.iter())
            .map(|(y, mu)| log_pdf_normal(*y, *mu, position[SIGMA]))
            .sum::<f64>();

        let logp = logp_y + logp_alpha + logp_beta + logp_sigma;

        // now the gradients -- d logp / d alpha, d logp / d beta, d logp / d sigma
        let d_alpha = self
            .y
            .iter()
            .zip(mu.iter())
            .map(|(y, mu)| (y - mu) / position[SIGMA].powi(2))
            .sum::<f64>();
        let d_beta = self
            .x
            .iter()
            .zip(self.y.iter())
            .zip(mu.iter())
            .map(|((x, y), mu)| (y - mu) * x / position[SIGMA].powi(2))
            .sum::<f64>();
        let d_sigma = self
            .y
            .iter()
            .zip(mu.iter())
            .map(|(y, mu)| (y - mu).powi(2) / position[SIGMA].powi(3) - 1.0 / position[SIGMA])
            .sum::<f64>();

        grad[ALPHA] = d_alpha;
        grad[BETA] = d_beta;
        grad[SIGMA] = d_sigma;

        Ok(logp)
    }

    fn dim(&self) -> usize {
        3
    }
}

impl Model for Regression {
    fn parameters(&self) -> Vec<String> {
        vec![
            String::from("alpha"),
            String::from("beta"),
            String::from("sigma"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand::SeedableRng;
    use rand_distr::Distribution;

    use crate::chain;

    use super::*;

    /// Run the regression.
    fn run_regression(
        x: Vec<f64>,
        y: Vec<f64>,
        chain_count: u64,
        tuning: u64,
        samples: u64,
        seed: u64,
        initial_position: Vec<f64>,
    ) -> Result<HashMap<String, Vec<Vec<f64>>>, RegressionError> {
        let model = Regression { x, y };
        assert_eq!(initial_position.len(), model.dim(), "Dimension mismatch");
        let chains =
            chain::Chains::run(seed, model, chain_count, tuning, samples, initial_position);

        let parameters = chains.parameters.clone();

        let mut ret = HashMap::new();
        for (i, parameter) in parameters.iter().enumerate() {
            ret.insert(parameter.clone(), chains.traces(i));
        }

        Ok(ret)
    }

    #[test]
    fn test_regression() {
        let x = vec![1., 2., 3., 4., 5.];
        let true_alpha = 2.;
        let true_beta = 3.;
        let true_sigma = 1.;

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(142);

        let noise = rand_distr::Normal::new(0., true_sigma).unwrap();
        let y = x
            .iter()
            .map(|x| true_alpha + true_beta * x + noise.sample(&mut rng))
            .collect::<Vec<_>>();

        let chain_count = 1;
        let tuning = 1000;
        let samples = 1000;
        let seed = 42;

        let guessed_alpha = y.iter().sum::<f64>() / y.len() as f64;
        let guessed_beta = y.iter().sum::<f64>() / x.iter().sum::<f64>();
        let guessed_sigma = 1.;
        let initial_position = vec![guessed_alpha, guessed_beta, guessed_sigma];

        let ret =
            run_regression(x, y, chain_count, tuning, samples, seed, initial_position).unwrap();

        assert_eq!(ret.len(), 3);
        assert_eq!(ret["alpha"].len(), chain_count as usize);
        assert_eq!(ret["beta"].len(), chain_count as usize);
        assert_eq!(ret["sigma"].len(), chain_count as usize);

        let alpha = ret["alpha"][0].iter().sum::<f64>() / samples as f64;
        let beta = ret["beta"][0].iter().sum::<f64>() / samples as f64;
        let sigma = ret["sigma"][0].iter().sum::<f64>() / samples as f64;

        println!("alpha: {}", alpha);
        println!("beta: {}", beta);
        println!("sigma: {}", sigma);
    }
}
