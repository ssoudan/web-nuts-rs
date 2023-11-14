//! Core logic

use nuts_rs::CpuLogpFunc;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;

use crate::{
    log,
    sampler::{be_nuts, MyDivergenceInfo},
};

#[derive(Default)]
pub struct Run {}

/// A model
pub(crate) trait Model: CpuLogpFunc {
    /// Return the names of the parameters
    fn parameters(&self) -> Vec<String>;
}

impl Run {
    fn run(
        &self,
        model: impl Model,
        seed: u64,
        tuning: u64,
        samples: u64,
        initial_position: Vec<f64>,
    ) -> ChainRun {
        let (trace, stats) = be_nuts(model, tuning, samples, &initial_position, seed);

        ChainRun { trace, stats }
    }
}

/// A single chain run.
struct ChainRun {
    trace: Vec<Box<[f64]>>,
    stats: Vec<MyDivergenceInfo>,
}

impl ChainRun {
    /// Return the trace for a given parameter.
    pub fn trace(&self, parameter_idx: usize) -> Vec<f64> {
        self.trace
            .iter()
            .map(|x| x[parameter_idx])
            .collect::<Vec<_>>()
    }

    /// Return the stats for divergences.
    #[allow(dead_code)]
    pub fn stats(&self) -> &Vec<MyDivergenceInfo> {
        &self.stats
    }
}

/// A collection of chains
pub(crate) struct Chains {
    chains: Vec<ChainRun>,
    dim: usize,
    pub(crate) parameters: Vec<String>,
}

impl Chains {
    /// Runs a collection of chains - sequentially.
    pub fn run(
        seed: u64,
        model: impl Model + Clone,
        chain_count: u64,
        tuning: u64,
        samples: u64,
        initial_position: Vec<f64>,
    ) -> Self {
        let chains = (0..chain_count)
            .map(|x| {
                Run::default().run(
                    model.clone(),
                    seed + x,
                    tuning,
                    samples,
                    initial_position.clone(),
                )
            })
            .collect();

        Chains {
            chains,
            dim: model.dim(),
            parameters: model.parameters(),
        }
    }

    /// Returns the extrema for a given parameter - across all chains.
    pub fn extrema(&self, parameter_idx: usize) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for chain in &self.chains {
            let (min_, max_) = chain
                .trace(parameter_idx)
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), x| {
                    (min.min(*x), max.max(*x))
                });

            min = min.min(min_);
            max = max.max(max_);
        }

        (min, max)
    }

    /// Returns the traces for a given parameter
    pub fn traces(&self, i: usize) -> Vec<Vec<f64>> {
        self.chains.iter().map(|x| x.trace(i)).collect()
    }

    pub(crate) fn plot(&self, canvas_id: &str, chains: &Chains, samples: u64) {
        let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        root.fill(&WHITE).unwrap();

        // split into DIMS horizontal subplots and 2 vertical subplots
        let subplots = root.split_evenly((self.dim, 2));

        let colors = [RED, GREEN, BLUE, MAGENTA, CYAN, YELLOW];

        let parameters = self.parameters.clone();

        // plot the histogram and traces
        for parameter_idx in 0..self.dim {
            let parameter = &parameters[parameter_idx];
            let (min_, max_) = chains.extrema(parameter_idx);

            let param_traces = chains.traces(parameter_idx);

            // ceil and floor at the nearest 0.1
            let (min_, max_) = ((min_ * 10.).floor() / 10., (max_ * 10.).ceil() / 10.);
            // let (min_, max_) = (min_.floor(), max_.ceil());

            log(format!(
                "parameter {}: min_ = {}, max_ = {}",
                parameter_idx, min_, max_
            )
            .as_str());
            // step size - about 10 bins between min_ and max_ - closest power of 10
            let step = 10.0f64.powf((max_ - min_).log10().floor() - 1.);

            // compute the height of the largest bin in the histogram
            let max_height = param_traces
                .iter()
                .map(|x| {
                    let mut counts = vec![0u32; ((max_ - min_) / step) as usize];
                    for x in x.iter() {
                        let idx = usize::min(((x - min_) / step) as usize, counts.len() - 1);
                        counts[idx] += 1;
                    }
                    counts.iter().copied().max().unwrap()
                })
                .max()
                .unwrap();

            // plot the histogram
            let root = &subplots[2 * parameter_idx];

            root.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(root)
                .margin(5)
                .caption(format!("Mu[{parameter}] (posterior)"), ("sans-serif", 30))
                .set_label_area_size(LabelAreaPosition::Left, 60)
                .set_label_area_size(LabelAreaPosition::Bottom, 30)
                .set_label_area_size(LabelAreaPosition::Right, 60)
                .build_cartesian_2d((min_..max_).step(step).use_round(), 0..max_height)
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .y_desc("Count")
                .y_label_style(TextStyle::from(("sans-serif", 20)).color(&BLACK))
                .x_label_style(TextStyle::from(("sans-serif", 20)).color(&BLACK))
                .draw()
                .unwrap();

            for (chain, param_trace) in param_traces.iter().enumerate() {
                let color = colors[chain % colors.len()];
                let style = color.mix(0.2).filled();

                let actual = Histogram::vertical(&chart)
                    .style(style)
                    .data(param_trace.iter().map(|x| (*x, 1)));

                chart
                    .draw_series(actual)
                    .unwrap()
                    .label(format!("Chain {chain}"))
                    .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], style));
            }
            chart.configure_series_labels().draw().unwrap();

            // plot the trace
            let mut chart = ChartBuilder::on(&subplots[2 * parameter_idx + 1])
                .margin(5)
                .caption(format!("Mu[{parameter}] (trace)"), ("sans-serif", 30))
                .x_label_area_size(30)
                .y_label_area_size(30)
                .set_label_area_size(LabelAreaPosition::Right, 60)
                .set_label_area_size(LabelAreaPosition::Bottom, 30)
                .build_cartesian_2d(0f64..samples as f64, min_..max_)
                .unwrap();

            chart
                .configure_mesh()
                .x_labels(3)
                .y_labels(3)
                .x_label_style(TextStyle::from(("sans-serif", 20)).color(&BLACK))
                .y_label_style(TextStyle::from(("sans-serif", 20)).color(&BLACK))
                .draw()
                .unwrap();

            for (chain, param_trace) in param_traces.iter().enumerate() {
                let color = colors[chain % colors.len()];

                chart
                    .draw_series(LineSeries::new(
                        (0..samples)
                            .zip(param_trace.iter())
                            .map(|(i, x)| (i as f64, *x)),
                        Into::<ShapeStyle>::into(color).stroke_width(1),
                    ))
                    .unwrap()
                    .label(format!("Chain {chain}"))
                    .legend(move |(x, y)| {
                        Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                    });
            }

            chart
                .configure_series_labels()
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK)
                .draw()
                .unwrap();
        }

        root.present().unwrap();
    }
}
