use std::u32;

use crate::*;

#[derive(Debug, Deserialize)]
pub struct DoubleHashDemoOpts {
    pub dark_mode: bool,
    #[serde(default)]
    pub reorient: bool,
    pub total_hashes: u32,
}

impl App {
    pub fn double_hash_demo(&self, mut options: DoubleHashDemoOpts) -> String {
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
            .chain(Some(u32::MAX))
            .map(|v| v as f64 / u32::MAX as f64)
            .collect_vec();

        let shift = hashes[0];
        hashes
            .iter_mut()
            .take(options.total_hashes as usize)
            .for_each(|h| *h -= shift);

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

        let (server_index_1, server_index_2) = if options.reorient {
            (
                0,
                getrandom::u64().unwrap() as usize
                    % (options.total_hashes as usize - 1)
                    + 1,
            )
        } else {
            loop {
                let (s1, s2) = (
                    getrandom::u64().unwrap() as usize
                        % (options.total_hashes as usize),
                    getrandom::u64().unwrap() as usize
                        % (options.total_hashes as usize),
                );
                if s1 != s2 {
                    break (s1.min(s2), s2.max(s1));
                }
            }
        };

        let server_left_1 = ticks[server_index_1];
        let server_right_1 = ticks[server_index_1 + 1];

        let server_left_2 = ticks[server_index_2];
        let server_right_2 = ticks[server_index_2 + 1];

        let size_1 = hashes[server_index_1 + 1] - hashes[server_index_1];
        let size_2 = hashes[server_index_2 + 1] - hashes[server_index_2];

        let expected_size = 2.0 / options.total_hashes as f64;
        let actual_size = size_1 + size_2;
        let error = (actual_size - expected_size) / expected_size;

        let actual_message = format!("{:.1}%", actual_size * 100.);

        let sum_message =
            format!("= {:.1}% + {:.1}%", size_1 * 100., size_2 * 100.,);

        let error_message = if error < 0. {
            format!("{:.1}% too small", -error * 100.)
        } else {
            format!("{:.1}% too big", error * 100.)
        };

        let server_rect_1 = rectangle(
            server_left_1,
            bar_top,
            server_right_1 - server_left_1,
            bar_height,
            GREEN,
        );

        let server_rect_2 = rectangle(
            server_left_2,
            bar_top,
            server_right_2 - server_left_2,
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

        let goal_text = text("", 100., 10., options.dark_mode)
            .set("text-align", "left")
            .set("font-size", FONT_SIZE * 0.75)
            .add(TSpan::new("Target: "))
            .add(
                TSpan::new(format!("{:.1}%", expected_size * 100.))
                    .set("font-weight", "bold"),
            );

        text_group = text_group.add(
            text("", 0., 0., options.dark_mode)
                .set("text-align", "left")
                .set("font-size", FONT_SIZE * 0.75)
                .set("text-anchor", "start")
                .add(
                    TSpan::new(&actual_message)
                        .set("font-weight", "bold")
                        .set("x", 0)
                        .set("dy", FONT_SIZE * 0.75),
                )
                .add(TSpan::new(&sum_message))
                .add(
                    TSpan::new(error_message)
                        .set("x", 0)
                        .set("dy", FONT_SIZE * 0.75),
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

        let server_arrow_1 = Path::new()
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
                        (server_left_1 + server_right_1) / 2.,
                        bar_top - 30.,
                        (server_left_1 + server_right_1) / 2.,
                        bar_top - 2.,
                    )),
            );

        let server_arrow_2 = Path::new()
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
                        (server_left_2 + server_right_2) / 2.,
                        bar_top - 30.,
                        (server_left_2 + server_right_2) / 2.,
                        bar_top - 2.,
                    )),
            );

        let document = Document::new()
            .set("viewBox", (0, 0, WIDTH, SHORT_HEIGHT))
            .add(defs)
            .add(demo.draw())
            .add(tick_group)
            .add(server_rect_1)
            .add(server_rect_2)
            .add(server_arrow_1)
            .add(server_arrow_2)
            .add(goal_text)
            .add(text_group);

        let mut e: Vec<u8> = Default::default();

        svg::write(&mut e, &document).expect("Failed to write data");

        String::from_utf8(e).unwrap()
    }
}
