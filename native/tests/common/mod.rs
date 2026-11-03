//! Shared by the tests: the tab started as Hangar would start it, the answers
//! Hangar would give (test/helpers.js's fake Hangar, as replies), and ways to
//! read the page that the harness does not have.
#![allow(dead_code, unused_imports)]

use serde_json::{Value, json};

use hangar_kit_sdk::testing::{self, Harness, StartExt};
use hangar_kit_sdk::ui::{self, Element, EventValue, HandlerId};
use hangar_kit_sdk::{KitError, Start, format};

pub use kit_annotate_tab::{Annotate, Phase};

/// The three permissions of the manifest.
pub const GRANTED: [&str; 3] = ["files.read", "files.write", "panes.send"];

/// The moment of the sends: 23:15:12 on 31 October 2026, in a time zone an
/// hour east of UTC, as the page's tests are at 23:15:12 local time.
pub fn now() -> i64 {
    format::parse_iso("2026-10-31T22:15:12Z").expect("a date")
}

pub fn start() -> Start {
    testing::set_now(Some(now()));
    testing::start().grant(GRANTED).utc_offset(60)
}

pub fn refusal(code: &str) -> Result<Value, KitError> {
    Err(KitError::new(code, format!("refused: {code}")))
}

/// Started on `path`, its size not answered yet.
pub fn asking(path: &str, start: Start) -> Harness<Annotate> {
    Harness::new(start.param("path", path))
}

/// Started on `path`, an 800 × 600 image.
pub fn opened(path: &str) -> Harness<Annotate> {
    opened_with(path, start())
}

pub fn opened_with(path: &str, start: Start) -> Harness<Annotate> {
    let mut tab = asking(path, start);
    tab.reply_to("image.size", Ok(json!({ "width": 800, "height": 600 })));
    tab
}

/// What `canvas.export` answers when it wrote the PNG at `scale`.
pub fn exported(path: &str, scale: f32) -> Result<Value, KitError> {
    let side = |size: f32| (size * scale).round() as u32;
    Ok(
        json!({ "path": path, "width": side(800.0), "height": side(600.0), "bytes": 12345, "scale": scale }),
    )
}

pub const PNG: &str = ".hangar/annotations/20261031-231512.png";
pub const JSON: &str = ".hangar/annotations/20261031-231512.json";

/// Every widget of the page, in order.
pub fn widgets(tab: &Harness<Annotate>) -> Vec<Element<HandlerId>> {
    let mut found = Vec::new();
    tab.view()
        .root
        .for_each(&mut |element| found.push(element.clone()));
    found
}

pub fn canvas(tab: &Harness<Annotate>) -> Option<ui::Canvas> {
    widgets(tab).into_iter().find_map(|element| match element {
        Element::Canvas(canvas) => Some(canvas),
        _ => None,
    })
}

pub fn buttons(tab: &Harness<Annotate>) -> Vec<ui::Button> {
    widgets(tab)
        .into_iter()
        .filter_map(|element| match element {
            Element::Button(button) => Some(button),
            _ => None,
        })
        .collect()
}

pub fn button(tab: &Harness<Annotate>, label: &str) -> Option<ui::Button> {
    buttons(tab).into_iter().find(|button| {
        button.label.as_deref() == Some(label) || button.tooltip.as_deref() == Some(label)
    })
}

pub fn filters(tab: &Harness<Annotate>) -> Vec<ui::Filter> {
    widgets(tab)
        .into_iter()
        .filter_map(|element| match element {
            Element::Filter(filter) => Some(filter),
            _ => None,
        })
        .collect()
}

pub fn swatches(tab: &Harness<Annotate>) -> Vec<ui::Swatch> {
    widgets(tab)
        .into_iter()
        .filter_map(|element| match element {
            Element::Swatch(swatch) => Some(swatch),
            _ => None,
        })
        .collect()
}

pub fn inputs(tab: &Harness<Annotate>) -> Vec<ui::TextInput> {
    widgets(tab)
        .into_iter()
        .filter_map(|element| match element {
            Element::TextInput(input) => Some(input),
            _ => None,
        })
        .collect()
}

pub fn input(tab: &Harness<Annotate>, id: &str) -> Option<ui::TextInput> {
    inputs(tab).into_iter().find(|input| input.id == id)
}

/// Types into the text input of this `id`: a note's field, which all share
/// their placeholder.
pub fn type_into_id(tab: &mut Harness<Annotate>, id: &str, text: &str) {
    let handler = input(tab, id)
        .and_then(|input| input.on_input)
        .unwrap_or_else(|| panic!("no field {id:?}"));
    tab.ui(handler, Some(EventValue::Text(text.to_owned())));
}

/// Hangar telling the canvas's scale, fitted to its room.
pub fn fit(tab: &mut Harness<Annotate>, scale: f64) {
    let handler = canvas(tab)
        .and_then(|canvas| canvas.on_fit)
        .expect("a fitted canvas");
    tab.ui(handler, Some(EventValue::Number(scale)));
}

/// Every word of the page a person may read: texts, labels, hints,
/// placeholders, tooltips.
pub fn words(tab: &Harness<Annotate>) -> Vec<String> {
    let mut words = tab.texts();
    for element in widgets(tab) {
        match element {
            Element::TextInput(input) => {
                words.push(input.placeholder.clone());
                words.extend(input.hint.clone());
            }
            Element::TextArea(area) => {
                words.extend(area.label.clone());
                words.push(area.placeholder.clone());
                words.extend(area.hint.clone());
            }
            Element::Tooltip(tooltip) => words.push(tooltip.text.clone()),
            Element::Swatch(swatch) => words.push(swatch.label.clone()),
            Element::Button(button) => words.extend(button.tooltip.clone()),
            _ => {}
        }
    }
    words
}

pub fn says(tab: &Harness<Annotate>, needle: &str) -> bool {
    words(tab).iter().any(|word| word.contains(needle))
}

/// The empty states of the page: `(kind, title, text)`.
pub fn empties(tab: &Harness<Annotate>) -> Vec<ui::Empty> {
    widgets(tab)
        .into_iter()
        .filter_map(|element| match element {
            Element::Empty(empty) => Some(empty),
            _ => None,
        })
        .collect()
}

pub fn banners(tab: &Harness<Annotate>) -> Vec<ui::Banner> {
    widgets(tab)
        .into_iter()
        .filter_map(|element| match element {
            Element::Banner(banner) => Some(banner),
            _ => None,
        })
        .collect()
}

/// Sending, as a trait so the tests read `tab.press_send()`: the primary
/// button, whatever it says.
pub trait Sending {
    fn press_send(&mut self);
}

impl Sending for Harness<Annotate> {
    fn press_send(&mut self) {
        let label = buttons(self)
            .into_iter()
            .find(|button| button.kind == ui::ButtonKind::Primary)
            .and_then(|button| button.label)
            .expect("a primary button");
        self.press(&label);
    }
}
