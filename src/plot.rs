//! Plot data
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;

/// Plot TMAX as a function of time
pub(crate) struct TMaxPlot {
    observed: Vec<Vec<f64>>,
}

impl TMaxPlot {
    /// Create a new plot
    pub(crate) fn new(observed: Vec<Vec<f64>>, parameters: Vec<String>) -> Self {
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0], "DATE");
        assert_eq!(parameters[1], "TMAX");

        Self { observed }
    }

    /// Plot the data
    pub fn plot(&self, canvas_id: &str) {
        let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
        let root = backend.into_drawing_area();

        root.fill(&WHITE).unwrap();

        let (date_min, date_max) = self
            .observed
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), x| {
                (min.min(x[0]), max.max(x[0]))
            });

        let (t_max_min, t_max_max) = self
            .observed
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), x| {
                (min.min(x[1]), max.max(x[1]))
            });

        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .caption("TMax", ("sans-serif", 30))
            .x_label_area_size(30)
            .y_label_area_size(40)
            .set_label_area_size(LabelAreaPosition::Right, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 30)
            .build_cartesian_2d(date_min..date_max, t_max_min..t_max_max)
            .unwrap();

        chart
            .configure_mesh()
            .x_labels(3)
            .y_labels(3)
            .x_label_style(TextStyle::from(("sans-serif", 20)).color(&BLACK))
            .y_label_style(TextStyle::from(("sans-serif", 20)).color(&BLACK))
            .draw()
            .unwrap();

        let observed = self.observed.clone();

        chart
            // .draw_series(LineSeries::new(
            // observed.iter().map(|d_t| (d_t[0], d_t[1])),
            // Into::<ShapeStyle>::into(RED).stroke_width(1),
            .draw_series(
                observed
                    .iter()
                    .map(|d_t| (d_t[0], d_t[1]))
                    .map(|(x, y)| Circle::new((x, y), 2, RED.filled())),
            )
            .unwrap()
            .label("TMax")
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], RED.filled()));

        chart.configure_series_labels().draw().unwrap();

        root.present().unwrap();
    }
}
