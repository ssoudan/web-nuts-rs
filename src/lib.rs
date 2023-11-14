mod chain;
mod model;

mod plot;
mod sampler;
mod utils;

use core::fmt;

use model::regression::Regression;

use utils::{download, set_panic_hook};
use wasm_bindgen::prelude::*;
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug)]
pub enum MyError {
    UnexpectedRawDataHeader,
    InvalidDateFormat,
}

impl std::error::Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::UnexpectedRawDataHeader => write!(f, "Unexpected raw data header"),
            MyError::InvalidDateFormat => write!(f, "Invalid date format - expected YYYYMMDD"),
        }
    }
}

impl From<MyError> for JsValue {
    fn from(val: MyError) -> Self {
        JsValue::from_str(&format!("{}", val))
    }
}

fn parse_csv(input_data: String) -> (Vec<Vec<f64>>, Vec<String>) {
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

    (observed, parameters)
}

/// Returns the date as a float representing the time in years
/// The input date is a string in the format YYYYMMDD.
fn parse_date(date: &str) -> Result<f64, MyError> {
    let year = date[0..4].parse::<i32>().unwrap();
    let month = date[4..6].parse::<u32>().unwrap();
    let day = date[6..8].parse::<u32>().unwrap();

    let date =
        chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or(MyError::InvalidDateFormat)?;

    let epoch = chrono::NaiveDate::from_ymd_opt(0, 1, 1).unwrap();

    let duration = date.signed_duration_since(epoch);
    let duration = duration.num_seconds() as f64;

    Ok(duration / (365.25 * 24.0 * 60.0 * 60.0))
}

#[wasm_bindgen]
pub fn prepare(raw_data: String) -> Result<String, MyError> {
    // receive data as CSV with the following header:
    // ID,DATE,ELEMENT,DATA_VALUE,M_FLAG,Q_FLAG,S_FLAG,OBS_TIME
    const EXPECTED_HEADER: &str = "ID,DATE,ELEMENT,DATA_VALUE,M_FLAG,Q_FLAG,S_FLAG,OBS_TIME";

    let raw_data = raw_data.trim();
    let lines: Vec<_> = raw_data.split('\n').collect();
    let header = lines[0];

    if header != EXPECTED_HEADER {
        return Err(MyError::UnexpectedRawDataHeader);
    }

    let mut output = String::new();
    // the output header is: DATE,TMAX
    output.push_str("DATE,TMAX\n");

    for line in lines.iter().skip(1) {
        let line = line.trim();
        let fields: Vec<_> = line.split(',').collect();
        let date = fields[1];
        let element = fields[2];
        let data_value = fields[3];
        let q_flag = fields[5];

        if element == "TMAX" && q_flag.is_empty() {
            // convert the date to years (float) since EPOCH
            let date = parse_date(date)?;
            let data_value = data_value.parse::<i32>().unwrap() as f64 / 10.0;

            output.push_str(format!("{},{}\n", date, data_value).as_str());
        }
    }

    Ok(output)
}

#[wasm_bindgen]
pub fn plot_tmax(canvas_id: &str, input_data: String) {
    set_panic_hook();

    let (observed, parameters) = parse_csv(input_data);

    let p = plot::TMaxPlot::new(observed, parameters);

    p.plot(canvas_id);
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
    log("Running");

    let (observed, _parameters) = parse_csv(input_data);

    // TODO(ssoudan) text
    // TODO(ssoudan) progress bar and dynamic status

    // let model = MultivariateNormalModel {
    //     observed,
    //     dims: parameters.len(),
    //     parameters,
    // };
    // let initial_position = vec![0.0; model.dim()];
    let x = observed.iter().map(|x| x[0]).collect::<Vec<_>>();
    let y = observed.iter().map(|x| x[1]).collect::<Vec<_>>();

    let model = Regression::new(x.clone(), y.clone());

    // y = alpha + beta * x + noise
    let guessed_beta = y.iter().sum::<f64>() / x.iter().sum::<f64>();
    let guessed_alpha = y.iter().sum::<f64>() / y.len() as f64;
    let guessed_sigma = x
        .iter()
        .zip(y.iter())
        .map(|(x, y)| (y - guessed_alpha - guessed_beta * x).powi(2))
        .sum::<f64>()
        .sqrt()
        / y.len() as f64;
    let initial_position = vec![guessed_alpha, guessed_beta, guessed_sigma];
    log(format!("initial_position = {:?}", initial_position).as_str());

    let chains = chain::Chains::run(seed, model, chain_count, tuning, samples, initial_position);

    log("Plotting");

    chains.plot(canvas_id, &chains, samples);

    log("Done");
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub async fn get_data(url: String) -> String {
    let data = download(url).await;
    data.unwrap().as_string().unwrap()
}
