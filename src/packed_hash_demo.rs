use std::u32;

use crate::*;

#[derive(Debug, Deserialize)]
pub struct PackedHashDemoOpts {
    pub dark_mode: bool,
    pub total_hashes: u32,
    pub summed_hashes: u32,
}

impl App {
    pub fn packed_hash_demo(&self, mut options: PackedHashDemoOpts) -> String {
        let demo = NumberLineDemo::new(options.dark_mode);

        if options.summed_hashes > options.total_hashes {
            options.summed_hashes = options.total_hashes;
        }

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

        let k = options.summed_hashes as usize;
        let servers_left = ticks[0];
        let servers_right = ticks[k];

        let actual_size = hashes[k] - hashes[0];

        let expected_size = k as f64 / options.total_hashes as f64;
        let error = (actual_size - expected_size) / expected_size;

        let actual_message = format!("{:.1}%", actual_size * 100.);

        let error_message = if error < 0. {
            format!("{:.1}% too small", -error * 100.)
        } else {
            format!("{:.1}% too big", error * 100.)
        };

        let server_rect_1 = rectangle(
            servers_left,
            bar_top,
            servers_right - servers_left,
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

        let servers_arrow_1 = Path::new()
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
                        (servers_left + servers_right) / 2.,
                        bar_top - 30.,
                        (servers_left + servers_right) / 2.,
                        bar_top - 2.,
                    )),
            );

        let document = Document::new()
            .set("viewBox", (0, 0, WIDTH, SHORT_HEIGHT))
            .add(defs)
            .add(demo.draw())
            .add(server_rect_1)
            .add(tick_group)
            .add(servers_arrow_1)
            .add(goal_text)
            .add(text_group);

        let mut e: Vec<u8> = Default::default();

        svg::write(&mut e, &document).expect("Failed to write data");

        String::from_utf8(e).unwrap()
    }
}
