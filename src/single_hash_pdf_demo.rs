use crate::*;

impl App {
    pub(crate) fn single_hash_pdf_demo(
        &mut self,
        mut options: SingleHashPdfDemo,
    ) -> String {
        options.histogram_bins = 1 << options.histogram_bins;
        options.total_hashes = options.total_hashes.max(2);
        options.sample_count = 1 << options.sample_count;

        let n = (1 << options.total_hashes) as f64;
        let mean = 1. / n;
        let std_dev = (2. / (n * (n + 1.)) - 1. / n.powi(2)).sqrt();

        let x_max = (mean + std_dev * 6.).min(1.);

        let config = ChConfig {
            measurement_count: options.sample_count as usize,
            bins: options.histogram_bins as usize,
            n: n as usize,
            k: 1,
            x_min: 0.0,
            x_max,
        };

        let expected = if options.actual_pdf {
            (0..=config.bins)
                .map(|i| {
                    let x = (i as f64 / config.bins as f64) * x_max;
                    let y = single_segment_pdf(x, config.n);
                    (x, y)
                })
                .map(|(x, y)| (x * 100., y))
                .collect_vec()
        } else {
            calculate_segment_histogram(&config, single_segment_cdf)
        };

        let mut y_max =
            expected.first().copied().unwrap_or((0., 0.0001)).1 * 1.1;

        let histogram_opt =
            options.run_simulation.then(|| simulate_histogram(&config));

        let actual = histogram_opt.as_ref().map(|histogram| {
            let actual_max = histogram.max * 100.;
            if actual_max > y_max {
                y_max = actual_max * 1.1;
            }

            histogram
                .histogram_fractions
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    (i as f64 / config.bins as f64 * x_max * 100., f * 100.)
                })
                .collect_vec()
        });

        let y_label = if options.actual_pdf {
            "Single Segment Length - PDF".to_owned()
        } else {
            "Histogram Band Probability (%)".to_owned()
        };

        draw_plot(PlotOptions {
            title: options.title,
            dark_mode: options.dark_mode,
            x_range: (0., x_max * 100.),
            y_range: (0., y_max),
            x_label: "Single Segment Length (%)".to_owned(),
            y_label,
            step: !options.actual_pdf,
            data_1: expected,
            data_2: actual,
        })
    }
}
