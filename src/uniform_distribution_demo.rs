use crate::*;

#[derive(Debug, Deserialize)]
pub struct UniformDistributionOptions {
    dark_mode: bool,
    x: f64,
    total_hashes: u32,
}

impl App {
    pub fn uniform_distribution_demo(
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
}
