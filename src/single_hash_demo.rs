use crate::*;

#[derive(Debug, Deserialize)]
pub struct SingleHashDemoOpts {
    pub dark_mode: bool,
    #[serde(default)]
    pub reorient: bool,
    pub total_hashes: u32,
}

impl App {
    pub fn single_hash_demo(&self, mut options: SingleHashDemoOpts) -> String {
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
}
