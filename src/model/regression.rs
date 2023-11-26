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
///
/// The model is a Bayesian regression model with a normal likelihood and
/// normal priors on the intercept and slope. The standard deviation of the
/// Gaussian has a flat prior.
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

fn log_pdf_normal_propto(diff: f64, log_sigma: f64, var_inv: f64) -> f64 {
    let norm = -log_sigma;
    let b = -0.5 * diff * diff * var_inv;
    norm + b
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

        let alpha = position[ALPHA];
        let beta = position[BETA];
        let sigma = position[SIGMA];

        let logp_alpha = log_pdf_normal_propto(alpha, 10f64.ln(), 0.01);
        let logp_beta = log_pdf_normal_propto(beta, 10f64.ln(), 0.01);
        let logp_sigma = 0.; // flat prior

        let mut d_logp_d_alpha = 0.;
        let mut d_logp_d_beta = 0.;
        let mut d_logp_d_sigma = 0.;

        let mut logp_y = 0.;

        let sigma_inv = sigma.recip();
        let var_inv = (sigma * sigma).recip();
        let var_sigma_inv = var_inv * sigma_inv;
        let log_sigma = sigma.ln();
        for (x, y) in self.x.iter().zip(self.y.iter()) {
            let mu_ = alpha + beta * x;
            let diff = y - mu_;

            logp_y += log_pdf_normal_propto(diff, log_sigma, var_inv);

            d_logp_d_alpha += diff * var_inv;
            d_logp_d_beta += diff * x * var_inv;
            d_logp_d_sigma += diff * diff * var_sigma_inv - sigma_inv;
        }

        let logp = logp_y + logp_alpha + logp_beta + logp_sigma;

        grad[ALPHA] = d_logp_d_alpha;
        grad[BETA] = d_logp_d_beta;
        grad[SIGMA] = d_logp_d_sigma;

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
