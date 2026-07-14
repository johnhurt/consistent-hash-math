use crate::*;

#[derive(Debug, Deserialize)]
pub struct KHashPdfDemo {
    dark_mode: bool,
    total_hashes: u32,
    summed_hashes: u32,
    histogram_bins: u32,
    sample_count: u32,
    title: String,

    #[serde(default)]
    run_simulation: bool,

    #[serde(default)]
    show_simulated_mean: bool,

    #[serde(default)]
    show_simulated_std_dev: bool,

    #[serde(default)]
    show_calculated_mean: bool,

    #[serde(default)]
    show_calculated_std_dev: bool,
}

/// Build a histogram from the k-segment PDF approximation by integrating each
/// bin with the trapezoid rule. This mirrors the approach in ch-math.
fn k_hash_expected_histogram(config: &ChConfig) -> Vec<(f64, f64)> {
    let width = config.x_max - config.x_min;

    if config.k == config.n {
        // Degenerate case: the server owns every hash, so the total length is
        // exactly 1.0. Put all probability mass in the last bin.
        let mut hist = vec![(0.0, 0.0); config.bins];
        if let Some(last) = hist.last_mut() {
            *last = (
                config.x_min
                    + width * (config.bins - 1) as f64 / config.bins as f64
                        * 100.0,
                100.0,
            );
        }
        return hist;
    }

    (0..config.bins)
        .map(|i| {
            let left = config.x_min + i as f64 / config.bins as f64 * width;
            let right =
                config.x_min + (i + 1) as f64 / config.bins as f64 * width;

            let p = if config.k == 1 {
                single_segment_cdf(right, config.n)
                    - single_segment_cdf(left, config.n)
            } else {
                let eps = 1e-12;
                let left_pdf =
                    k_segment_pdf_approx(left.max(eps), config.n, config.k);
                let right_pdf = k_segment_pdf_approx(
                    right.min(1.0 - eps),
                    config.n,
                    config.k,
                );
                (left_pdf + right_pdf) * (right - left) / 2.0
            };

            (left * 100.0, p * 100.0)
        })
        .collect()
}

impl App {
    pub(crate) fn k_hash_pdf_demo(
        &mut self,
        mut options: KHashPdfDemo,
    ) -> String {
        options.histogram_bins = 1 << options.histogram_bins;
        options.sample_count = 1 << options.sample_count;
        options.total_hashes = options.total_hashes.max(2);
        options.summed_hashes = options.summed_hashes.max(1);

        if options.summed_hashes > options.total_hashes {
            options.summed_hashes = options.total_hashes;
        }

        let n = options.total_hashes as usize;
        let k = options.summed_hashes as usize;

        let (x_min, x_max) = if k == n {
            (0.0, 1.0)
        } else {
            let mean = k as f64 / n as f64;
            let variance = k as f64 * (n - k) as f64
                / ((n as f64).powi(2) * (n + 1) as f64);
            let std_dev = variance.sqrt();
            (
                (mean - std_dev * 6.0).max(0.0),
                (mean + std_dev * 6.0).min(1.0),
            )
        };

        let config = ChConfig {
            measurement_count: options.sample_count as usize,
            bins: options.histogram_bins as usize,
            n,
            k,
            x_min,
            x_max,
        };

        let expected = k_hash_expected_histogram(&config);

        let mut y_max =
            expected.iter().map(|(_, y)| *y).fold(0.0001, f64::max) * 1.1;

        let histogram_opt =
            options.run_simulation.then(|| simulate_histogram(&config));

        let width = x_max - x_min;
        let mut vertical_bands = Vec::new();

        let actual = histogram_opt.as_ref().map(|histogram| {
            let actual_max = histogram.max * 100.0;
            if actual_max > y_max {
                y_max = actual_max * 1.1;
            }

            if options.show_simulated_mean || options.show_simulated_std_dev {
                let mean = histogram.mean * 100.0;
                let lower =
                    (histogram.mean - histogram.std_dev).max(0.0) * 100.0;
                let upper = (histogram.mean + histogram.std_dev) * 100.0;

                let band = VerticalBand::new(lower, mean, upper, BLUE_LIGHT)
                    .with_edge_dash("4,4")
                    .with_mean_dash("12,6")
                    .with_mean_line_color(BLUE_SATURATED)
                    .with_mean_line_width(2.5);
                vertical_bands.push(band);

            }

            histogram
                .histogram_fractions
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    (
                        (x_min + i as f64 / config.bins as f64 * width) * 100.0,
                        f * 100.0,
                    )
                })
                .collect_vec()
        });

        if options.show_calculated_mean || options.show_calculated_std_dev {
            let mean = k as f64 / n as f64 * 100.0;
            let variance = k as f64 * (n - k) as f64
                / ((n as f64).powi(2) * (n + 1) as f64);
            let std_dev = variance.sqrt() * 100.0;
            let lower = (mean - std_dev).max(0.0);
            let upper = mean + std_dev;

            let band = VerticalBand::new(lower, mean, upper, RED_LIGHT)
                .with_edge_dash("6,6")
                .with_mean_dash("16,8")
                .with_mean_line_color(RED_SATURATED)
                .with_mean_line_width(2.5);
            vertical_bands.push(band);
        }

        draw_plot(PlotOptions {
            title: options.title,
            dark_mode: options.dark_mode,
            x_range: (x_min * 100.0, x_max * 100.0),
            y_range: (0.0, y_max),
            x_label: "k-Hash Sum Length (%)".to_owned(),
            y_label: "Histogram Band Probability (%)".to_owned(),
            step: true,
            data_1: expected,
            data_2: actual,
            data_1_color: Some(RED_SATURATED),
            data_2_color: Some(BLUE_SATURATED),
            vertical_bands,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(n: usize, k: usize) -> ChConfig {
        let mean = k as f64 / n as f64;
        let variance =
            k as f64 * (n - k) as f64 / ((n as f64).powi(2) * (n + 1) as f64);
        let std_dev = variance.sqrt();
        let x_min = (mean - std_dev * 6.0).max(0.0);
        let x_max = (mean + std_dev * 6.0).min(1.0);
        ChConfig {
            measurement_count: 10000,
            bins: 64,
            n,
            k,
            x_min,
            x_max,
        }
    }

    #[test]
    fn check_expected_histogram_total() {
        for (n, k) in [
            (20, 3),
            (20, 10),
            (20, 19),
            (100, 10),
            (100, 20),
            (100, 50),
            (100, 90),
            (213, 200),
            (213, 212),
            (20, 20),
            (100, 100),
            (1000, 100),
            (1000, 500),
            (1000, 900),
        ] {
            let config = test_config(n, k);
            let hist = k_hash_expected_histogram(&config);
            let total: f64 = hist.iter().map(|(_, p)| p / 100.0).sum();
            println!(
                "n={} k={} x_min={:.4} x_max={:.4} integrated={:.4}",
                n, k, config.x_min, config.x_max, total
            );
            assert!(
                total > 0.8 && total < 1.2,
                "histogram total {} is outside plausible range",
                total
            );
        }
    }

    #[test]
    fn check_simulation_mean() {
        for (n, k) in
            [(20, 3), (20, 19), (20, 20), (100, 10), (100, 50), (100, 90)]
        {
            let config = test_config(n, k);
            let hist = simulate_histogram(&config);
            println!(
                "simulated n={} k={} mean={:.4} expected={:.4}",
                n,
                k,
                hist.mean,
                config.k as f64 / config.n as f64
            );
        }
    }
}
