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
        AxisConfig, GridConfig, LabelConfig, LegendConfig, TickConfig,
        TitleConfig,
    },
};
use serde::Deserialize;
use std::{
    f64::consts::PI,
    mem::{self, swap},
    panic,
};
use svg::{
    node::element::{
        path::Data, Definitions, Group, Marker, Path, Rectangle, TSpan, Text,
    },
    Document,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::console_log;

const WIDTH: f64 = 600.;
const SHORT_HEIGHT: f64 = 100.;
const TALL_HEIGHT: f64 = 500.;
const MARGIN: f64 = 20.;
const FONT_SIZE: f64 = 15.;

pub const BLACK: &str = "#0d1117";

pub const GREEN: &str = "#c0ffc0";
pub const BLUE: &str = "#c0c0ff";
pub const RED: &str = "#ffc0c0";

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
struct SingleHashDemoOpts {
    dark_mode: bool,
    #[serde(default)]
    reorient: bool,
    total_hashes: u32,
}

#[derive(Debug, Deserialize)]
struct UniformDistributionOptions {
    dark_mode: bool,
    x: f64,
    total_hashes: u32,
}

#[derive(Debug, Deserialize)]
struct SingleHashPdfDemo {
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

struct ChConfig {
    n: usize,
    k: usize,

    measurement_count: usize,
    bins: usize,
    x_max: f64,
}

#[derive(Debug)]
struct HistogramOutput {
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

/// Shuffle the values in the given array
fn shuffle<T>(v: &mut [T]) {
    generate_random_ints(v.len())
        .into_iter()
        .enumerate()
        .sorted_by_key(|(_, k)| *k)
        .enumerate()
        .for_each(|(i, (j, _))| v.swap(i, j));
}

/// Generate a list of random u32s
fn generate_random_ints(length: usize) -> Vec<u32> {
    let mut bytes = vec![0u8; length * 4];
    getrandom::fill(&mut bytes).unwrap();

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
fn generate_random_floats(length: usize) -> Vec<f64> {
    let ints = generate_random_ints(length);
    ints.into_iter()
        .sorted()
        .map(|v| v as f64 / u32::MAX as f64)
        .collect_vec()
}

/// Perform the base layer of the experiment by generating a bunch of hashes
/// (random numbers) and returning a list of all the sizes of all the segments
fn generate_segments(count: usize) -> Vec<ordered_float::OrderedFloat<f64>> {
    generate_random_floats(count)
        .into_iter()
        .circular_tuple_windows::<(_, _)>()
        .map(|(left, mut right)| {
            if left > right {
                right += 1.0
            }

            OrderedFloat::from(right - left)
        })
        .collect()
}

fn sum_k_segments(
    segments: &[OrderedFloat<f64>],
    k: usize,
) -> Vec<OrderedFloat<f64>> {
    segments
        .iter()
        .take(segments.len() / k)
        .chunks(k)
        .into_iter()
        .map(|chunk| chunk.into_iter().sum())
        .collect()
}

fn simulate_histogram(config: &ChConfig) -> HistogramOutput {
    let bins = config.bins;
    let mut histogram = vec![0_u32; bins];
    let max = config.x_max;
    let mut samples = 0;
    let mean = config.k as f64 / config.n as f64;

    let mut sum = 0.;
    let mut running_variance = 0.;

    while samples < config.measurement_count {
        let mut new_segments = generate_segments(config.n);

        // Shuffle the segment lengths to
        shuffle(&mut new_segments);

        let segment_sums = sum_k_segments(&new_segments, config.k);

        for &segment_sum in &segment_sums {
            sum += segment_sum.0;
            running_variance += (segment_sum.0 - mean).powi(2);
            let index = (segment_sum.0 / max * bins as f64) as usize;

            if let Some(v) = histogram.get_mut(index) {
                *v += 1;
            }

            samples += 1;

            if samples >= config.measurement_count {
                break;
            }
        }
    }
    let samples = samples as f64;

    let max =
        histogram.iter().copied().max().unwrap_or_default() as f64 / samples;

    HistogramOutput {
        histogram_fractions: histogram
            .into_iter()
            .map(|c| c as f64 / samples)
            .collect(),
        mean: sum / samples,
        std_dev: (running_variance / samples).sqrt(),
        max,
        samples,
    }
}

/// This is an approximation to the pdf of the sum of k segments lengths out of
/// n total segments. The real formula is
///
/// binomial(n - 1, k - 1) * (n - k) * (1-x)^(n - k - 1) * x^(k - 1)
///
/// This approximation uses the stirling approximation for factorials
///
fn k_segment_pdf_approx(at: f64, n: usize, k: usize) -> f64 {
    let n = n as f64;
    let k = k as f64;

    console_log!("at: {at}, n: {n}, k: {k}");

    // Terms 1-3 are the stirling approximation applied to the
    // binomial coefficient of (n - 1, k - 1)
    let t_1 = 0.5 * ((n - 1.) / (2. * PI * (k - 1.) * (n - k))).ln();
    let t_2 = (k - 1.) * ((n - 1.) / (k - 1.)).ln();
    let t_3 = (n - k) * ((n - 1.) / (n - k)).ln();

    let t_4 = (n - k).ln();
    let t_5 = (n - k - 1.) * (1. - at).ln();
    let t_6 = (k - 1.) * at.ln();

    (t_1 + t_2 + t_3 + t_4 + t_5 + t_6).exp()
}

/// This is the pdf function for setups where k = 1. It exists because the k-
/// segment approximation blows up at k = 1 because it requires taking a log of
/// zero
fn single_segment_pdf(at: f64, n: usize) -> f64 {
    (n as f64 - 1.) * (1. - at).powi(n as i32 - 2)
}

fn single_segment_cdf(at: f64, n: usize) -> f64 {
    1. - (1. - at).powi(n as i32 - 1)
}

fn calculate_segment_histogram<F: FnMut(f64, usize) -> f64>(
    config: &ChConfig,
    max: f64,
    mut cdf: F,
) -> Vec<(f64, f64)> {
    (0..=config.bins)
        .chain(Some(config.bins))
        .tuple_windows::<(_, _)>()
        .map(|(left_i, right_i)| {
            let left = left_i as f64 / config.bins as f64 * max;
            let right = right_i as f64 / config.bins as f64 * max;

            let left_v = cdf(left, config.n);
            let right_v = cdf(right, config.n);

            let p = right_v - left_v;

            (left * 100., p * 100.)
        })
        .collect()
}

#[derive(Debug, Clone)]
struct NumberLineDemo {
    bar_bottom: f64,
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
}

fn draw_plot(options: PlotOptions) -> String {
    use Interpolation as I;

    let (fg_color, mid_color) = if options.dark_mode {
        ("white", "darkgray")
    } else {
        (BLACK, "lightgray")
    };

    let s1 = Series::builder()
        .name("Expected")
        .data(options.data_1.clone())
        .color(RED)
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
            .color(BLUE)
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
            min: options.x_range.0,
            max: options.x_range.1,
        })
        .y_range(Range::Manual {
            min: options.y_range.0,
            max: options.y_range.1,
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
            bar_bottom,
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

impl App {
    fn uniform_distribution_demo(
        &self,
        mut options: UniformDistributionOptions,
    ) -> String {
        let demo = NumberLineDemo::new(options.dark_mode);

        let NumberLineDemo {
            bar_top,
            bar_height,
            bar_left,
            bar_right,
            bar_width,
            text_center_y,
            little_tick_top,
            little_tick_height,
            big_tick_top,
            big_tick_height,
            // foreground_color,
            // background_color,
            ..
        } = demo.clone();

        let x = options.x / 100.;
        let x_x = x * bar_width + bar_left;

        let x_text_x = x_x.clamp(bar_left + 10., bar_right - 10.);

        let x_tick = tick(x_x, big_tick_top, big_tick_height);
        let x_text = text(
            "x",
            x_text_x,
            text_center_y + FONT_SIZE / 2.,
            options.dark_mode,
        );

        let x_bar = rectangle(x_x, bar_top, bar_right - x_x, bar_height, GREEN);

        options.total_hashes = 1 << options.total_hashes;

        if options.total_hashes < 1 {
            options.total_hashes = 1;
        }

        let hashes = generate_random_floats(options.total_hashes as usize);

        let above_count = hashes.iter().copied().filter(|&v| v > x).count();

        let result = above_count as f64 / options.total_hashes as f64;

        let ticks = hashes
            .iter()
            .copied()
            .map(|v| v * (WIDTH - 2. * MARGIN) + MARGIN)
            .collect_vec();

        let mut tick_group = Group::new();

        if options.total_hashes < 1000 {
            for x in &ticks {
                tick_group = tick_group.add(tick(
                    *x,
                    little_tick_top,
                    little_tick_height,
                ));
            }
        } else {
            tick_group = tick_group.add(rectangle(
                bar_left,
                little_tick_top,
                bar_width,
                little_tick_height,
                "gray",
            ));
        }

        let mut text_group = Group::new().set(
            "transform",
            format!("translate({},{})", WIDTH / 2., MARGIN / 10.),
        );

        let small_font = FONT_SIZE * 0.9;

        text_group = text_group.add(
            text("", 0., 0., options.dark_mode)
                .set("text-align", "right")
                .set("font-size", small_font)
                .set("text-anchor", "end")
                .add(TSpan::new("1 - x = ").set("x", 0).set("dy", small_font))
                .add(TSpan::new(format!("{:.1}%", (1. - x) * 100.)))
                .add(TSpan::new("% > x = ").set("x", 0).set("dy", small_font))
                .add(TSpan::new(format!("{:0.1}%", result * 100.))),
        );

        let document = Document::new()
            .set("viewBox", (0, 0, WIDTH, SHORT_HEIGHT))
            .add(demo.draw())
            .add(x_tick)
            .add(x_bar)
            .add(x_text)
            .add(tick_group)
            .add(text_group);

        let mut e: Vec<u8> = Default::default();

        svg::write(&mut e, &document).expect("Failed to write data");

        String::from_utf8(e).unwrap()
    }

    fn single_hash_demo(&self, mut options: SingleHashDemoOpts) -> String {
        let demo = NumberLineDemo::new(options.dark_mode);

        let NumberLineDemo {
            bar_top,
            bar_height,
            little_tick_top,
            little_tick_height,
            foreground_color,
            background_color,
            ..
        } = demo.clone();

        let text_width = 100.;

        if options.total_hashes < 2 {
            options.total_hashes = 2;
        }

        let mut hashes = (0..options.total_hashes)
            .map(|_| getrandom::u32().unwrap())
            .sorted()
            .map(|v| v as f64 / u32::MAX as f64)
            .collect_vec();

        if options.reorient {
            let shift = hashes[0];
            hashes.iter_mut().for_each(|h| *h -= shift);
        }

        let ticks = hashes
            .iter()
            .copied()
            .map(|v| v * (WIDTH - 2. * MARGIN) + MARGIN)
            .collect_vec();

        let mut tick_group = Group::new();

        for x in &ticks {
            tick_group =
                tick_group.add(tick(*x, little_tick_top, little_tick_height));
        }

        let server_index = if options.reorient {
            0
        } else {
            getrandom::u64().unwrap() as usize
                % (options.total_hashes as usize - 1)
        };

        let server_left = ticks[server_index];
        let server_right = ticks[server_index + 1];

        let expected_size = 1.0 / options.total_hashes as f64;
        let actual_size = hashes[server_index + 1] - hashes[server_index];
        let error = (actual_size - expected_size) / expected_size;

        let message = if error < 0. {
            format!("{:.1}% too small", -error * 100.)
        } else {
            format!("{:.1}% too big", error * 100.)
        };

        let server_rect = rectangle(
            server_left,
            bar_top,
            server_right - server_left,
            bar_height,
            GREEN,
        );

        let text_right_x = WIDTH / 2.0;
        let mut text_group = Group::new().set(
            "transform",
            format!("translate({},{})", text_right_x, MARGIN / 2.),
        );

        text_group = text_group.add(
            rectangle(
                -5.,
                0.,
                text_width,
                2. * FONT_SIZE * 0.75 * 1.05,
                background_color,
            )
            .set("stroke", "none"),
        );

        text_group = text_group.add(
            text("", 0., 0., options.dark_mode)
                .set("text-align", "left")
                .set("font-size", FONT_SIZE * 0.75)
                .set("text-anchor", "start")
                .add(
                    TSpan::new("Our Server")
                        .set("x", 0)
                        .set("dy", FONT_SIZE * 0.75),
                )
                .add(
                    TSpan::new(message).set("x", 0).set("dy", FONT_SIZE * 0.75),
                ),
        );

        let defs = Definitions::new().add(
            Marker::new()
                .set("id", "arrow-head")
                .set("orient", "auto")
                .set("markerWidth", 6)
                .set("markerHeight", 8)
                .set("refX", 3)
                .set("refY", 4)
                .add(
                    Path::new().set("fill", foreground_color).set(
                        "d",
                        Data::new()
                            .move_to((0, 0))
                            .vertical_line_by(8)
                            .line_to((4, 4))
                            .close(),
                    ),
                ),
        );

        let server_arrow = Path::new()
            .set("marker-end", "url(#arrow-head)")
            .set("stroke_width", 1)
            .set("stroke", foreground_color)
            .set("fill", "none")
            .set(
                "d",
                Data::new()
                    .move_to((
                        text_right_x + text_width / 2.,
                        MARGIN / 2. + FONT_SIZE * 0.75,
                    ))
                    .quadratic_curve_to((
                        (server_left + server_right) / 2.,
                        bar_top - 30.,
                        (server_left + server_right) / 2.,
                        bar_top - 2.,
                    )),
            );

        let document = Document::new()
            .set("viewBox", (0, 0, WIDTH, SHORT_HEIGHT))
            .add(defs)
            .add(demo.draw())
            .add(tick_group)
            .add(server_rect)
            .add(server_arrow)
            .add(text_group);

        let mut e: Vec<u8> = Default::default();

        svg::write(&mut e, &document).expect("Failed to write data");

        String::from_utf8(e).unwrap()
    }

    fn single_hash_pdf_demo(
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
            calculate_segment_histogram(&config, x_max, single_segment_cdf)
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

#[wasm_bindgen]
pub fn eval_message(app: &mut App, id: String, options: String) -> String {
    match id.as_str() {
        "single-hash-demo" => app.single_hash_demo(
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
        _ => "Unknown id".to_owned(),
    }

    // let data = Data::new()
    //     .move_to((1, 1))
    //     .line_by((98, 0))
    //     .line_by((0, 8))
    //     .line_by((-98, 0))
    //     .close();

    // let path = Path::new()
    //     .set("fill", "none")
    //     .set("stroke", "blue")
    //     .set("stroke-width", 1)
    //     .set("d", data);

    // let document = Document::new().set("viewBox", (0, 0, 100, 10)).add(path);

    // let mut e: Vec<u8> = Default::default();

    // svg::write(&mut e, &document).expect("Failed to write data");

    // console_log!("done");

    // String::from_utf8(e).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_uniform_dist_demo() {
        let options = UniformDistributionOptions {
            x: 20.,
            total_hashes: 6,
            dark_mode: false,
        };

        let a = App {};

        let f = generate_random_floats(1);

        let _ = a.uniform_distribution_demo(options);
    }
}
