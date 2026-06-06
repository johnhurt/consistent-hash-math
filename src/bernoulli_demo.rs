use crate::*;

#[derive(Debug, Deserialize)]
pub struct BernoulliDemoOptions {
    dark_mode: bool,
    x: f64,
    total_hashes: u32,
}

impl App {
    pub fn bernoulli_demo(&self, mut options: BernoulliDemoOptions) -> String {
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

        // Winners (< x) are green on the left, losers (>= x) are red on the right
        let win_bar =
            rectangle(bar_left, bar_top, x_x - bar_left, bar_height, GREEN);
        let lose_bar =
            rectangle(x_x, bar_top, bar_right - x_x, bar_height, RED);

        options.total_hashes = 1 << options.total_hashes;

        if options.total_hashes < 1 {
            options.total_hashes = 1;
        }

        let hashes = generate_random_floats(options.total_hashes as usize);

        let winner_count = hashes.iter().copied().filter(|&v| v < x).count();
        let total = options.total_hashes as usize;
        let actual_pct = winner_count as f64 / total as f64 * 100.;

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
                .add(
                    TSpan::new(format!(
                        "Winners: {} / {} \u{2248} {:.1}%",
                        winner_count, total, actual_pct
                    ))
                    .set("x", 0)
                    .set("dy", small_font),
                )
                .add(
                    TSpan::new(format!("Expected: {:.1}%", x * 100.))
                        .set("x", 0)
                        .set("dy", small_font),
                ),
        );

        let document = Document::new()
            .set("viewBox", (0, 0, WIDTH, SHORT_HEIGHT))
            .add(demo.draw())
            .add(win_bar)
            .add(lose_bar)
            .add(x_tick)
            .add(x_text)
            .add(tick_group)
            .add(text_group);

        let mut e: Vec<u8> = Default::default();
        svg::write(&mut e, &document).expect("Failed to write data");
        String::from_utf8(e).unwrap()
    }
}
