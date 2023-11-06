mod model;
mod sampler;
mod utils;

use nuts_rs::CpuLogpFunc;

use plotters::prelude::*;
use plotters_canvas::CanvasBackend;

use sampler::{be_nuts, MyDivergenceInfo};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Default)]
pub struct Run {}

impl Run {
    fn run(&self, model: model::Model, seed: u64, tuning: u64, samples: u64) -> ChainRun {
        log(format!("seed={}", seed).as_str());
        let (trace, stats) = be_nuts(model, tuning, samples, seed);

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
    pub fn trace(&self, i: usize) -> Vec<f64> {
        self.trace.iter().map(|x| x[i]).collect::<Vec<_>>()
    }

    /// Return the stats for divergences.
    #[allow(dead_code)]
    pub fn stats(&self) -> &Vec<MyDivergenceInfo> {
        &self.stats
    }
}

/// A collection of chains
struct Chains {
    chains: Vec<ChainRun>,
    dim: usize,
    parameters: Vec<String>,
}

impl Chains {
    /// Runs a collection of chains - sequentially.
    pub fn run(
        seed: u64,
        model: model::Model,
        chain_count: u64,
        tuning: u64,
        samples: u64,
    ) -> Self {
        let chains = (0..chain_count)
            .map(|x| Run::default().run(model.clone(), seed + x, tuning, samples))
            .collect();

        Chains {
            chains,
            dim: model.dim(),
            parameters: model.parameters.clone(),
        }
    }

    /// Returns the extrema for a given parameter - across all chains.
    pub fn extrema(&self, i: usize) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for chain in &self.chains {
            let (min_, max_) = chain
                .trace(i)
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

    fn plot(&self, canvas_id: &str, chains: &Chains, samples: u64) {
        let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        root.fill(&WHITE).unwrap();

        // split into DIMS horizontal subplots and 2 vertical subplots
        let subplots = root.split_evenly((self.dim, 2));

        let colors = [RED, GREEN, BLUE, MAGENTA, CYAN, YELLOW];

        let parameters = self.parameters.clone();

        // plot the histogram and traces
        for row in 0..self.dim {
            let parameter = &parameters[row];
            let (min_, max_) = chains.extrema(row);

            let param_traces = chains.traces(row);

            // ceil and floor
            let (min_, max_) = (min_.floor(), max_.ceil());
            // step size
            let step = 0.1;

            // compute the height of the largest bin in the histogram
            let max_height = param_traces
                .iter()
                .map(|x| {
                    let mut counts = vec![0u32; ((max_ - min_) / step) as usize];
                    for x in x.iter() {
                        let idx = ((x - min_) / step) as usize;
                        counts[idx] += 1;
                    }
                    counts.iter().copied().max().unwrap()
                })
                .max()
                .unwrap();

            // plot the histogram
            let root = &subplots[2 * row];

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
            let mut chart = ChartBuilder::on(&subplots[2 * row + 1])
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

#[wasm_bindgen]
pub fn run_with(
    canvas_id: &str,
    seed: u64,
    input_data: String,
    chain_count: u64,
    tuning: u64,
    samples: u64,
) {
    set_panic_hook();

    let input_data = input_data.trim();
    let lines: Vec<_> = input_data.split('\n').collect();
    let headers = lines[0];
    let parameters = headers
        .split(',')
        .map(|x| x.trim().to_string())
        .collect::<Vec<_>>();

    let observed = lines
        .iter()
        .skip(1)
        .map(|x| {
            let x_ = x
                .split(',')
                .map(|x| x.trim().parse::<f64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(x_.len(), parameters.len(), "Wrong number of columns");
            x_
        })
        .collect::<Vec<_>>();

    let model = model::Model {
        observed,
        dims: parameters.len(),
        parameters,
    };
    let chains = Chains::run(seed, model, chain_count, tuning, samples);

    chains.plot(canvas_id, &chains, samples)
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

