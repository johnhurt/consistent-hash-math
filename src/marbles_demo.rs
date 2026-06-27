use aho_corasick::AhoCorasick;
use compact_str::{format_compact, CompactString};
use std::{collections::HashMap, sync::LazyLock};
use svg::{
    node::{
        self,
        element::{tag::Type, Element},
    },
    parser::Event,
    Parser,
};

use crate::*;

static TEMPLATE: &str = include_str!("resources/marbles.svg");
static REPLACER: LazyLock<AhoCorasick> = LazyLock::new(|| {
    AhoCorasick::new([
        "RED",
        "GREEN",
        "BLACK",
        "{red_red}",
        "{red_green}",
        "{green_red}",
        "{green_green}",
        "{same}",
        "{different}",
        "{actual}",
        "{expected}",
    ])
    .expect("All good here")
});

struct Replacements {
    red: &'static str,
    green: &'static str,
    black: &'static str,
    red_red: f64,
    red_green: f64,
    green_red: f64,
    green_green: f64,
    same: f64,
    different: f64,
    actual: f64,
    expected: f64,
}

impl Replacements {
    fn into_vec(self) -> Vec<CompactString> {
        vec![
            self.red.into(),
            self.green.into(),
            self.black.into(),
            format_compact!("{:.01}%", self.red_red * 100.),
            format_compact!("{:.01}%", self.red_green * 100.),
            format_compact!("{:.01}%", self.green_red * 100.),
            format_compact!("{:.01}%", self.green_green * 100.),
            format_compact!("{:.01}%", self.same * 100.),
            format_compact!("{:.01}%", self.different * 100.),
            format_compact!("{:.01}%", self.actual * 100.),
            format_compact!("{:.01}%", self.expected * 100.),
        ]
    }
}

#[derive(Debug, Deserialize)]
pub struct MarblesDemoOpts {
    color_count: u32,
    trial_count: u32,
    dark_mode: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum RedGreen {
    Red,
    Green,
}

impl RedGreen {
    fn from_random(rand: u32, total_red: u32, total_green: u32) -> Self {
        let mod_rand = rand % (total_red + total_green);

        if mod_rand < total_red {
            RedGreen::Red
        } else {
            RedGreen::Green
        }
    }
}

/// Why am I having to write this?
#[allow(dead_code)]
fn parse_doc_into<'l>(curr: &mut Element, parser: &mut Parser<'l>) {
    while let Some(e) = parser.next() {
        match e {
            Event::Tag(name, Type::Start, attrs) => {
                let mut child = Element::new(name);
                child.get_attributes_mut().extend(attrs);
                parse_doc_into(&mut child, parser);
                curr.get_children_mut().push(Box::new(child));
            }
            Event::Tag(name, Type::Empty, attrs) => {
                let mut child = Element::new(name);
                child.get_attributes_mut().extend(attrs);
                curr.get_children_mut().push(Box::new(child));
            }
            Event::Tag(_, Type::End, _) => {
                return;
            }
            Event::Text(t) => {
                curr.get_children_mut().push(Box::new(node::Text::new(t)));
            }
            _ => {}
        }
    }
}

fn render_svg(replacements: Replacements) -> String {
    REPLACER.replace_all(TEMPLATE, &replacements.into_vec())
}

#[allow(dead_code)]
fn template_svg(dark_mode: bool) -> Document {
    let fg_color = if dark_mode { WHITE } else { BLACK };
    let template_svg = TEMPLATE
        .replace("#000000", fg_color)
        .replace("#FF0000", RED)
        .replace("#00FF00", GREEN);
    let mut parser = Parser::new(&template_svg);
    let mut result = Document::new();

    let Some(Event::Tag("svg", _, attrs)) = parser.next() else {
        panic!("expected svg tag");
    };

    result.get_attributes_mut().extend(attrs);

    parse_doc_into(&mut result, &mut parser);

    result
}

/// Evaluate a single 2-marble trial without sampling
fn single_marble_trial(
    rand_1: u32,
    rand_2: u32,
    count_per_color: u32,
) -> (RedGreen, RedGreen) {
    use RedGreen as RG;
    let mut total_red = count_per_color;
    let mut total_green = count_per_color;

    let first = RedGreen::from_random(rand_1, total_red, total_green);
    match first {
        RG::Red => total_red -= 1,
        RG::Green => total_green -= 1,
    }
    let second = RG::from_random(rand_2, total_red, total_green);
    (first, second)
}

impl App {
    pub fn marbles_demo(&mut self, options: MarblesDemoOpts) -> String {
        use RedGreen as RG;
        let mut results = HashMap::<(RedGreen, RedGreen), usize>::new();
        generate_random_ints(options.trial_count as usize * 2)
            .iter()
            .tuples::<(_, _)>()
            .map(|(first, second)| {
                single_marble_trial(*first, *second, options.color_count)
            })
            .for_each(|key| {
                results.entry(key).and_modify(|c| *c += 1).or_insert(1);
            });

        let total = options.trial_count as f64;

        let get_result = |first, second| {
            results.get(&(first, second)).copied().unwrap_or_default() as f64
        };

        let red_red = get_result(RG::Red, RG::Red);
        let red_green = get_result(RG::Red, RG::Green);
        let green_red = get_result(RG::Green, RG::Red);
        let green_green = get_result(RG::Green, RG::Green);
        let same = red_red + green_green;
        let different = red_green + green_red;
        let actual = same / different;
        let expected =
            (options.color_count as f64 - 1.) / options.color_count as f64;

        let replacements = Replacements {
            red: RED,
            green: GREEN,
            black: if options.dark_mode { WHITE } else { BLACK },
            red_red: red_red / total,
            red_green: red_green / total,
            green_red: green_red / total,
            green_green: green_green / total,
            same: same / total,
            different: different / total,
            actual,
            expected,
        };

        render_svg(replacements)
    }
}

#[test]
fn d() {
    template_svg(true);
}
