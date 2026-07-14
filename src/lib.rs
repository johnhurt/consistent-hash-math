mod bernoulli_demo;
mod double_hash_demo;
mod k_hash_pdf_demo;
mod kth_hash_demo;
mod marbles_demo;
mod packed_hash_demo;
mod single_hash_demo;
mod single_hash_pdf_demo;
mod uniform_distribution_demo;
mod utils;

use core::str;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use quill::{
    color::Color,
    plot::Plot,
    prelude::{Grid, Interpolation, Legend, Line, Range, Scale},
    series::Series,
    style::{
        AxisConfig, GridConfig, LabelConfig, Margin, TickConfig, TitleConfig,
    },
};
use serde::Deserialize;
use std::{f64::consts::PI, mem, panic};
use svg::node::element::{
    path::Data, Definitions, Group, Marker, Path, Rectangle, TSpan, Text,
};
pub(crate) use svg::Document;
use svg_legacy::node::element::{Line as SvgLine, Rectangle as SvgRect};
use wasm_bindgen::prelude::*;

use crate::single_hash_demo::SingleHashDemoOpts;

const WIDTH: f64 = 600.;
const SHORT_HEIGHT: f64 = 100.;
const TALL_HEIGHT: f64 = 500.;
const MARGIN: f64 = 20.;
const FONT_SIZE: f64 = 15.;

pub const BLACK: &str = "#0d1117";
pub const WHITE: &str = "#ffffff";

pub const GREEN: &str = "#c0ffc0";
pub const BLUE: &str = "#c0c0ff";
pub const BLUE_LIGHT: &str = "#8080ff";

pub const RED: &str = "#ffc0c0";
pub const RED_LIGHT: &str = "#ff8080";

pub const RED_SATURATED: &str = "#ff5c5c";
pub const BLUE_SATURATED: &str = "#5c5cff";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub enum DataType {
    Cloudflare,
    Other,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct App {}

#[wasm_bindgen]
pub fn new_app() -> App {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::default()
}

#[derive(Debug, Deserialize)]
pub(crate) struct SingleHashPdfDemo {
    dark_mode: bool,
    total_hashes: u32,

    sample_count: u32,

    histogram_bins: u32,
    title: String,

    #[serde(default)]
    actual_pdf: bool,

    #[serde(default)]
    run_simulation: bool,
}

pub(crate) struct ChConfig {
    n: usize,
    k: usize,

    measurement_count: usize,
    bins: usize,
    x_min: f64,
    x_max: f64,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct HistogramOutput {
    histogram_fractions: Vec<f64>,
    samples: f64,
    mean: f64,
    std_dev: f64,
    max: f64,
}

fn rectangle(x: f64, y: f64, width: f64, height: f64, fill: &str) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", width)
        .set("height", height)
        .set("fill", fill)
        .set("stroke", "gray")
        .set("stroke-width", 0.5)
        .set("stroke-linecap", "round")
        .set("stroke-linejoin", "round")
}

fn tick(x: f64, y: f64, height: f64) -> Rectangle {
    rectangle(x, y, 0.1, height, "gray")
}

fn text(text: &str, x: f64, y: f64, dark_mode: bool) -> Text {
    Text::new(text)
        .set("x", x)
        .set("y", y)
        .set("fill", if dark_mode { "white" } else { BLACK })
        .set("font-size", FONT_SIZE)
        .set("text-anchor", "middle")
        .set("alignment-baseline", "middle")
}

/// Generate a list of random u32s
pub fn generate_random_ints(length: usize) -> Vec<u32> {
    let mut bytes = vec![0u8; length * 4];
    getrandom::fill(&mut bytes).unwrap();

    // This is safe because it's essentially just reinterpreting the random
    // bytes as u32s simply to save copying and extra allocations
    unsafe {
        let capacity = bytes.capacity() / 4;
        let ptr = bytes.as_mut_ptr() as *mut u32;
        mem::forget(bytes);
        Vec::from_raw_parts(ptr, length, capacity)
    }
}

/// Generate a sorted array of the given number random floats between 0 and 1
pub(crate) fn generate_random_floats(length: usize) -> Vec<f64> {
    let ints = generate_random_ints(length);
    ints.into_iter()
        .sorted()
        .map(|v| v as f64 / u32::MAX as f64)
        .collect_vec()
}

/// Batched generator that produces one ring's worth of segments at a time.
///
/// Instead of calling `getrandom` for every run, it fills a ~1 MB buffer of
/// random u32s once and then sorts slices of that buffer into segment rings
/// until the buffer is exhausted.
struct SegmentGenerator {
    n: usize,
    batch: Vec<u32>,
    floats: Vec<f64>,
    segments: Vec<OrderedFloat<f64>>,
    used: usize,
}

impl SegmentGenerator {
    fn new(n: usize) -> Self {
        let per_run = n.saturating_sub(1).max(1);
        let target_bytes = 1_000_000;
        let batch_runs = (target_bytes / (per_run * 4)).max(1);
        let batch = vec![0; batch_runs * per_run];

        Self {
            n,
            batch,
            floats: Vec::with_capacity(per_run + 1),
            segments: Vec::with_capacity(n),
            used: batch_runs * per_run,
        }
    }

    fn next(&mut self) -> &[OrderedFloat<f64>] {
        let per_run = self.n.saturating_sub(1).max(1);

        if self.used.saturating_add(per_run) > self.batch.len() {
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    self.batch.as_mut_ptr() as *mut u8,
                    self.batch.len() * 4,
                )
            };
            getrandom::fill(bytes).unwrap();
            self.used = 0;
        }

        self.floats.clear();
        self.floats.extend(
            self.batch[self.used..self.used + per_run]
                .iter()
                .map(|&v| v as f64 / u32::MAX as f64),
        );
        self.floats.push(0.0);
        self.floats.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.segments.clear();
        for window in self.floats.windows(2) {
            self.segments
                .push(OrderedFloat::from(window[1] - window[0]));
        }
        let wrap = (1.0 + self.floats[0]) - self.floats[self.floats.len() - 1];
        self.segments.push(OrderedFloat::from(wrap));

        self.used += per_run;
        &self.segments
    }
}

fn sum_k_segments(
    segments: &[OrderedFloat<f64>],
    k: usize,
) -> Vec<OrderedFloat<f64>> {
    segments
        .iter()
        .take((segments.len() / k) * k)
        .chunks(k)
        .into_iter()
        .map(|chunk| chunk.into_iter().sum())
        .collect()
}

pub(crate) fn simulate_histogram(config: &ChConfig) -> HistogramOutput {
    let bins = config.bins;
    let min = config.x_min;
    let max = config.x_max;
    let width = max - min;

    let mut histogram = vec![0_u32; bins];
    let mut samples = 0;
    let mut sample_mean = 0.0;
    let mut m2 = 0.0;

    let mut generator = SegmentGenerator::new(config.n);

    while samples < config.measurement_count {
        let needed = config.measurement_count - samples;

        let new_segments = generator.next();
        let segment_sums = sum_k_segments(new_segments, config.k);

        for &segment_sum in
            segment_sums.iter().take(needed.min(segment_sums.len()))
        {
            samples += 1;
            let x = segment_sum.0;
            let delta = x - sample_mean;
            sample_mean += delta / samples as f64;
            let delta2 = x - sample_mean;
            m2 += delta * delta2;

            let raw_index = if width > 0.0 {
                (x - min) / width * bins as f64
            } else {
                bins as f64 - 1.0
            };

            if raw_index >= 0.0 {
                let index = (raw_index.min(bins as f64 - 1.0)) as usize;
                histogram[index] += 1;
            }
        }
    }

    let samples_f = samples as f64;

    let max =
        histogram.iter().copied().max().unwrap_or_default() as f64 / samples_f;

    let std_dev = if samples > 1 {
        (m2 / (samples_f - 1.0)).sqrt()
    } else {
        0.0
    };

    HistogramOutput {
        histogram_fractions: histogram
            .into_iter()
            .map(|c| c as f64 / samples_f)
            .collect(),
        mean: sample_mean,
        std_dev,
        max,
        samples: samples_f,
    }
}

/// PDF for the length of k adjacent segments out of n total segments on the
/// unit circle.
///
/// This matches the article's formula directly:
///
///   binomial(n - 1, k) * k * x^(k - 1) * (1 - x)^(n - 1 - k)
///
/// The binomial coefficient is approximated with Stirling's formula.
pub(crate) fn k_segment_pdf_approx(at: f64, n: usize, k: usize) -> f64 {
    if k == 0 {
        return 0.0;
    }

    // k == n is the degenerate case where the entire ring is always taken.
    if k == n {
        return if (at - 1.0).abs() < f64::EPSILON {
            f64::INFINITY
        } else {
            0.0
        };
    }

    let n_f = n as f64;
    let k_f = k as f64;
    let nf = n_f - 1.0;
    let nmk = nf - k_f; // n - 1 - k

    let log_choose = ln_choose_stirling(nf, k_f);
    let log_k = k_f.ln();

    // When the exponent is zero, the value of the power term is 1 even if the
    // base is at a boundary (0 or 1), so we skip the log to avoid 0 * -inf.
    let log_x = if k <= 1 { 0.0 } else { (k_f - 1.0) * at.ln() };
    let log_1mx = if nmk <= 0.0 {
        0.0
    } else {
        nmk * (1.0 - at).ln()
    };

    (log_choose + log_k + log_x + log_1mx).exp()
}

/// ln(C(n, k)) via Stirling's approximation. Returns 0 for the boundary cases
/// C(n, 0) and C(n, n), whose value is 1.
fn ln_choose_stirling(n: f64, k: f64) -> f64 {
    if k <= 0.0 || k >= n {
        return 0.0;
    }

    let nmk = n - k;
    0.5 * (n / (2.0 * PI * k * nmk)).ln()
        + k * (n / k).ln()
        + nmk * (n / nmk).ln()
}

/// This is the pdf function for setups where k = 1. It exists because the k-
/// segment approximation blows up at k = 1 because it requires taking a log of
/// zero
fn single_segment_pdf(at: f64, n: usize) -> f64 {
    (n as f64 - 1.) * (1. - at).powi(n as i32 - 2)
}

pub(crate) fn single_segment_cdf(at: f64, n: usize) -> f64 {
    1. - (1. - at).powi(n as i32 - 1)
}

pub(crate) fn calculate_segment_histogram<F: FnMut(f64, usize) -> f64>(
    config: &ChConfig,
    mut cdf: F,
) -> Vec<(f64, f64)> {
    let width = config.x_max - config.x_min;
    (0..=config.bins)
        .chain(Some(config.bins))
        .tuple_windows::<(_, _)>()
        .map(|(left_i, right_i)| {
            let t_left = left_i as f64 / config.bins as f64;
            let t_right = right_i as f64 / config.bins as f64;
            let left = config.x_min + t_left * width;
            let right = config.x_min + t_right * width;

            let left_v = cdf(left, config.n);
            let right_v = cdf(right, config.n);

            let p = right_v - left_v;

            (left * 100., p * 100.)
        })
        .collect()
}

#[derive(Debug, Clone)]
struct NumberLineDemo {
    bar_top: f64,
    bar_height: f64,

    bar_left: f64,
    bar_right: f64,
    bar_width: f64,

    text_center_y: f64,
    big_tick_top: f64,
    little_tick_top: f64,
    big_tick_height: f64,
    little_tick_height: f64,

    foreground_color: &'static str,
    background_color: &'static str,
}

struct PlotOptions {
    title: String,
    dark_mode: bool,
    x_range: (f64, f64),
    y_range: (f64, f64),
    x_label: String,
    y_label: String,
    step: bool,
    data_1: Vec<(f64, f64)>,
    data_2: Option<Vec<(f64, f64)>>,
    data_1_color: Option<&'static str>,
    data_2_color: Option<&'static str>,
    vertical_bands: Vec<VerticalBand>,
}

struct VerticalBand {
    lower: f64,
    mean: f64,
    upper: f64,
    color: &'static str,
    show_fill: bool,
    edge_dash_array: &'static str,
    mean_dash_array: Option<&'static str>,
    mean_line_color: Option<&'static str>,
    mean_line_width: f64,
}

impl VerticalBand {
    fn new(lower: f64, mean: f64, upper: f64, color: &'static str) -> Self {
        Self {
            lower,
            mean,
            upper,
            color,
            show_fill: true,
            edge_dash_array: "4,4",
            mean_dash_array: None,
            mean_line_color: None,
            mean_line_width: 2.0,
        }
    }

    fn with_edge_dash(mut self, dash: &'static str) -> Self {
        self.edge_dash_array = dash;
        self
    }

    fn with_mean_dash(mut self, dash: &'static str) -> Self {
        self.mean_dash_array = Some(dash);
        self
    }

    fn with_mean_line_color(mut self, color: &'static str) -> Self {
        self.mean_line_color = Some(color);
        self
    }

    fn with_mean_line_width(mut self, width: f64) -> Self {
        self.mean_line_width = width;
        self
    }
}

fn draw_plot(options: PlotOptions) -> String {
    use Interpolation as I;

    let (fg_color, mid_color) = if options.dark_mode {
        ("white", "darkgray")
    } else {
        (BLACK, "lightgray")
    };

    let data_1_color = options.data_1_color.unwrap_or(RED);
    let data_2_color = options.data_2_color.unwrap_or(BLUE);

    let (x_min, x_max) = options.x_range;
    let (y_min, y_max) = options.y_range;

    let s1 = Series::builder()
        .name("Expected")
        .data(options.data_1.clone())
        .color(data_1_color)
        .line(Line::Solid)
        .interpolation(if options.step { I::Step } else { I::Linear })
        .line_width(3.)
        .build();

    let mut legend = Legend::None;
    let s2 = if let Some(data_2) = options.data_2 {
        legend = Legend::TopRightInside;
        Series::builder()
            .name("Simulated")
            .data(data_2)
            .color(data_2_color)
            .line(Line::Solid)
            .interpolation(if options.step { I::Step } else { I::Linear })
            .line_width(3.)
            .build()
    } else {
        Series::builder().name("Empty").data(vec![]).build()
    };

    let data = [s1, s2];

    let plot = Plot::builder()
        .dimensions((WIDTH as i32, TALL_HEIGHT as i32))
        .title(&options.title)
        .grid_config(GridConfig {
            color: Color::from(fg_color),
            minor_color: Color::from(mid_color),
            ..Default::default()
        })
        .legend(legend)
        .x_label(&options.x_label)
        .y_label(&options.y_label)
        .x_range(Range::Manual {
            min: x_min,
            max: x_max,
        })
        .y_range(Range::Manual {
            min: y_min,
            max: y_max,
        })
        .axis_config(AxisConfig {
            color: Color::from(fg_color),
            ..Default::default()
        })
        .x_label_config(LabelConfig {
            color: Color::from(fg_color),
            ..Default::default()
        })
        .y_label_config(LabelConfig {
            color: Color::from(fg_color),
            ..Default::default()
        })
        .title_config(TitleConfig {
            color: Color::from(fg_color),
            ..Default::default()
        })
        .tick_config(TickConfig {
            line_color: Color::from(fg_color),
            label_color: Color::from(fg_color),
            ..Default::default()
        })
        .x_scale(Scale::Scientific)
        .grid(Grid::Solid)
        .data(data);

    let mut doc = plot
        .build()
        .to_document()
        .expect("Failed to create plot svg");

    // Draw optional vertical bands (mean +/- std-dev) by injecting SVG shapes
    // directly into the plot area.
    for band in &options.vertical_bands {
        let margin = Margin::default();
        let plot_left = margin.left as f64;
        let plot_right = WIDTH - margin.right as f64;
        let plot_top = margin.top as f64;
        let plot_bottom = TALL_HEIGHT - margin.bottom as f64;
        let x_span = x_max - x_min;

        if x_span > 0.0 {
            let x_to_svg = |x: f64| {
                plot_left + (x - x_min) / x_span * (plot_right - plot_left)
            };

            let lower_x = x_to_svg(band.lower);
            let mean_x = x_to_svg(band.mean);
            let upper_x = x_to_svg(band.upper);

            // Semi-transparent fill between lower and upper bounds.
            if band.show_fill {
                let band_rect = SvgRect::new()
                    .set("x", lower_x)
                    .set("y", plot_top)
                    .set("width", upper_x - lower_x)
                    .set("height", plot_bottom - plot_top)
                    .set("fill", band.color)
                    .set("fill-opacity", 0.2)
                    .set("stroke", "none");
                doc = doc.add(band_rect);
            }

            // Dashed edge lines at the std-dev boundaries.
            for &x in &[lower_x, upper_x] {
                let edge = SvgLine::new()
                    .set("x1", x)
                    .set("y1", plot_top)
                    .set("x2", x)
                    .set("y2", plot_bottom)
                    .set("stroke", band.color)
                    .set("stroke-width", 1.5)
                    .set("stroke-dasharray", band.edge_dash_array);
                doc = doc.add(edge);
            }

            // Mean line (solid by default, dashed when requested).
            let mean_color = band.mean_line_color.unwrap_or(band.color);
            let mut mean_line = SvgLine::new()
                .set("x1", mean_x)
                .set("y1", plot_top)
                .set("x2", mean_x)
                .set("y2", plot_bottom)
                .set("stroke", mean_color)
                .set("stroke-width", band.mean_line_width);
            if let Some(dash) = band.mean_dash_array {
                mean_line = mean_line.set("stroke-dasharray", dash);
            }
            doc = doc.add(mean_line);
        }
    }

    // Quill doesn't allow you to set the background color, so we just remove
    // the background which is always the first child 😂
    doc.get_children_mut().remove(0);

    // Remove the height and width of the svg so that charts will render
    // correctly on mobile
    doc.get_attributes_mut()
        .retain(|name, _| !matches!(name.as_str(), "height" | "width"));

    doc.to_string()
}

impl NumberLineDemo {
    fn new(dark_mode: bool) -> Self {
        let bar_bottom = SHORT_HEIGHT - MARGIN;
        let bar_top = bar_bottom * 3. / 4.;
        let bar_height = bar_bottom - bar_top;

        let bar_left = MARGIN;
        let bar_right = WIDTH - MARGIN;
        let bar_width = bar_right - bar_left;

        let text_center_y = MARGIN + FONT_SIZE / 2.;

        let big_tick_top = MARGIN + FONT_SIZE * 1.5;
        let little_tick_top = MARGIN + FONT_SIZE * 2.;

        let big_tick_height = bar_bottom - big_tick_top;
        let little_tick_height = bar_bottom - little_tick_top;

        let (foreground_color, background_color) = if dark_mode {
            ("white", BLACK)
        } else {
            (BLACK, "white")
        };

        NumberLineDemo {
            bar_top,
            bar_height,

            bar_left,
            bar_right,
            bar_width,

            text_center_y,
            big_tick_top,
            little_tick_top,
            big_tick_height,
            little_tick_height,
            foreground_color,
            background_color,
        }
    }

    fn draw(&self) -> Group {
        Group::new()
            .add(rectangle(
                self.bar_left,
                self.bar_top,
                self.bar_width,
                self.bar_height,
                BLUE,
            ))
            .add(tick(MARGIN, self.big_tick_top, self.big_tick_height))
            .add(tick(
                WIDTH - MARGIN,
                self.big_tick_top,
                self.big_tick_height,
            ))
            .add(text("0", MARGIN, self.text_center_y, false))
            .add(text("1", WIDTH - MARGIN, self.text_center_y, false))
    }
}

#[wasm_bindgen]
pub fn eval_message(app: &mut App, id: String, options: String) -> String {
    match id.as_str() {
        "single-hash-demo" => app.single_hash_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "double-hash-demo-1" | "double-hash-demo-2" => app.double_hash_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "single-hash-demo-reorient" => {
            let mut options: SingleHashDemoOpts =
                serde_json::from_str(&options)
                    .expect("Failed to parse options");
            options.reorient = true;
            app.single_hash_demo(options)
        }
        "uniform-distribution-demo" => app.uniform_distribution_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "single-hash-pdf-demo"
        | "single-hash-pdf-demo-hist"
        | "single-hash-simulation-hist" => app.single_hash_pdf_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "marble-demo" => app.marbles_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "packed-hash-demo-1" | "packed-hash-demo-2" => app.packed_hash_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "kth-hash-demo" => app.kth_hash_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "bernoulli-demo" => app.bernoulli_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        "k-hash-pdf" | "k-hash-pdf-comparison" => app.k_hash_pdf_demo(
            serde_json::from_str(&options).expect("Failed to parse options"),
        ),
        _ => "Unknown id".to_owned(),
    }
}
