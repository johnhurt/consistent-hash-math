use std::u32;

use crate::*;

fn ordinal(n: usize) -> String {
    let suffix = match n % 100 {
        11 | 12 | 13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{}{}", n, suffix)
}

#[derive(Debug, Deserialize)]
pub struct KthHashDemoOpts {
    pub dark_mode: bool,
    pub total_hashes: u32,
    pub summed_hashes: u32,
}

impl App {
    pub fn kth_hash_demo(&self, mut options: KthHashDemoOpts) -> String {
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

        // Clamp inputs. summed_hashes == total_hashes is valid: the (k+1)-th
        // hash is the sentinel at 1.0, representing the wrap-around point.
        if options.total_hashes < 1 {
            options.total_hashes = 1;
        }
        if options.summed_hashes > options.total_hashes {
            options.summed_hashes = options.total_hashes;
        }

        let n = options.total_hashes as usize;
        let k = options.summed_hashes as usize;

        // Generate n hashes, sort them, pin first to zero by shifting
        let mut hashes = (0..options.total_hashes)
            .map(|_| getrandom::u32().unwrap())
            .sorted()
            .chain(Some(u32::MAX))
            .map(|v| v as f64 / u32::MAX as f64)
            .collect_vec();

        let shift = hashes[0];
        hashes.iter_mut().take(n).for_each(|h| *h -= shift);

        let ticks = hashes
            .iter()
            .copied()
            .map(|v| v * (WIDTH - 2. * MARGIN) + MARGIN)
            .collect_vec();

        // Draw all hash ticks
        let mut tick_group = Group::new();
        for x in &ticks {
            tick_group =
                tick_group.add(tick(*x, little_tick_top, little_tick_height));
        }

        // Draw a green box around each of the first k segments
        let mut segment_group = Group::new();
        for i in 0..k {
            let seg_left = ticks[i];
            let seg_right = ticks[i + 1];
            segment_group = segment_group.add(rectangle(
                seg_left,
                bar_top,
                seg_right - seg_left,
                bar_height,
                GREEN,
            ));
        }

        // The (k+1)-th hash (index k in the 0-based sorted array) is the target
        let target_tick_x = ticks[k];
        let sum_val = hashes[k]; // since h_1 is pinned at 0, sum of first k segments = h_{k+1}

        // Arrow definition
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

        // Label is centered in the panel
        let text_center_x = WIDTH / 2.;
        let line_gap = FONT_SIZE * 0.75 * 1.6;
        let text_width = 140.;
        let text_height = 2. * line_gap;
        let text_top = MARGIN / 2.;
        let font_size = FONT_SIZE * 0.75;

        // The arrow starts from the center of the text block and curves down
        // to the target tick. It is drawn first so the background rect (drawn
        // next) occludes the portion that overlaps the label, making it appear
        // to originate just outside the text.
        let arrow_root_x = text_center_x;
        let arrow_root_y = text_top + text_height / 2.;

        let arrow = Path::new()
            .set("marker-end", "url(#arrow-head)")
            .set("stroke-width", 1)
            .set("stroke", foreground_color)
            .set("fill", "none")
            .set(
                "d",
                Data::new()
                    .move_to((arrow_root_x, arrow_root_y))
                    .quadratic_curve_to((
                        target_tick_x,
                        bar_top - 30.,
                        target_tick_x,
                        bar_top - 2.,
                    )),
            );

        // Background rect drawn over the arrow but under the text, so the
        // arrow appears to start just at the edge of the label box.
        let label_bg = rectangle(
            text_center_x - text_width / 2.,
            text_top,
            text_width,
            text_height,
            background_color,
        )
        .set("stroke", "none");

        // Build the label.
        // Line 1: "{ordinal} hash = {value}"  (bold), or "All hashes = 1.000" when k == n
        // Line 2: "sum of k segment(s)"
        let hash_label = if k == n {
            format!("All hashes = {:.3}", sum_val)
        } else {
            format!("{} hash = {:.3}", ordinal(k + 1), sum_val)
        };

        let label = text("", text_center_x, text_top, options.dark_mode)
            .set("font-size", font_size)
            .set("text-anchor", "middle")
            .add(
                TSpan::new(hash_label)
                    .set("font-weight", "bold")
                    .set("x", text_center_x)
                    .set("dy", line_gap),
            )
            .add(
                TSpan::new(format!(
                    "sum of {} segment{}",
                    k,
                    if k == 1 { "" } else { "s" }
                ))
                .set("x", text_center_x)
                .set("dy", line_gap),
            );

        let document = Document::new()
            .set("viewBox", (0, 0, WIDTH, SHORT_HEIGHT))
            .add(defs)
            .add(demo.draw())
            .add(segment_group)
            .add(tick_group)
            .add(arrow)
            .add(label_bg)
            .add(label);

        let mut e: Vec<u8> = Default::default();
        svg::write(&mut e, &document).expect("Failed to write data");
        String::from_utf8(e).unwrap()
    }
}
