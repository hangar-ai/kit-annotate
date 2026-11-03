//! The words, the params, the names, the zoom.

use std::collections::BTreeSet;

use hangar_kit_sdk::format;
use hangar_kit_sdk::widgets::canvas;

use kit_annotate_tab::{answer, logic, model};

fn keys(table: &[(&str, &str)]) -> BTreeSet<String> {
    table.iter().map(|(key, _)| (*key).to_owned()).collect()
}

fn vars(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        found.push(after[..close].to_owned());
        rest = &after[close + 1..];
    }
    found.sort();
    found
}

#[test]
fn english_and_french_have_the_same_keys_and_french_keeps_its_accents() {
    assert_eq!(keys(logic::FR), keys(logic::EN));
    assert_eq!(keys(logic::EN).len(), logic::EN.len(), "no key twice");
    for (key, english) in logic::EN {
        let french = logic::FR.iter().find(|(other, _)| other == key).unwrap().1;
        assert_eq!(vars(french), vars(english), "{key}");
    }
    let fr = logic::strings("fr");
    assert_eq!(fr.t("empty_title"), "Pas encore d’image");
    assert_eq!(fr.t("tool_arrow"), "Flèche");
    assert_eq!(fr.t("tool_free"), "Main levée");
    assert!(fr.t("denied_text").contains("Réglages › Kits"));
}

#[test]
fn the_language_is_french_for_any_french_locale_english_otherwise() {
    for locale in ["fr", "fr-FR", "FR-ca", "fr_BE"] {
        assert_eq!(logic::pick_lang(locale), "fr", "{locale}");
    }
    for locale in ["en", "en-US", "de", "frisian", ""] {
        assert_eq!(logic::pick_lang(locale), "en", "{locale}");
    }
    assert_eq!(logic::strings("FR-ca").t("tab"), "Annoter");
}

#[test]
fn a_missing_key_or_variable_shows_itself_rather_than_nothing() {
    assert_eq!(logic::strings("en").t("nope"), "nope");
    assert_eq!(logic::strings("fr").t("loading"), "Lecture de {path}…");
    assert_eq!(
        logic::strings("en").tv("loading", &[("path", "a.png")]),
        "Reading a.png…"
    );
}

#[test]
fn the_path_param_a_short_string_else_nothing() {
    assert_eq!(
        logic::clean_path(Some(" .hangar/a.png ")).as_deref(),
        Some(".hangar/a.png")
    );
    let long = "x".repeat(1025);
    for bad in [None, Some(""), Some("   "), Some(long.as_str())] {
        assert_eq!(logic::clean_path(bad), None, "{bad:?}");
    }
    assert_eq!(logic::base_name(".hangar/a.png"), "a.png");
    assert_eq!(logic::base_name("sub\\b.png"), "b.png");
}

#[test]
fn a_name_is_the_local_time_and_a_taken_one_gets_a_counter() {
    let at = format::parse_iso("2026-10-31T22:05:09Z").unwrap();
    assert_eq!(logic::stamp(&format::local_time(at, 60)), "20261031-230509");
    assert_eq!(logic::stamp(&format::local_time(at, 0)), "20261031-220509");
    assert_eq!(
        logic::names("20261031-230509", 3),
        ["20261031-230509", "20261031-230509-2", "20261031-230509-3"]
    );
    assert_eq!(logic::iso(at + 7), "2026-10-31T22:05:09.007Z");
}

#[test]
fn only_bitmaps_are_offered_never_an_svg_or_a_pdf() {
    for extension in ["png", "jpg", "jpeg", "webp", "gif"] {
        assert!(logic::PICKABLE.contains(&extension), "{extension}");
    }
    for extension in ["svg", "pdf", "html"] {
        assert!(!logic::PICKABLE.contains(&extension), "{extension}");
    }
}

#[test]
fn fitting_never_enlarges_and_zoom_steps_move_by_one() {
    // Hangar fits the canvas itself (`Canvas::fit`): as large as the room
    // and 480 points allow, never above 100 %.
    let fit = |width: f32, height: f32, room: f32| canvas::<()>("shot", width, height).fit(room);
    assert_eq!(fit(1000.0, 500.0, 500.0), 0.5);
    assert_eq!(fit(1000.0, 4000.0, 500.0), 0.12);
    assert_eq!(fit(100.0, 100.0, 500.0), 1.0);
    assert_eq!(logic::step_zoom(0.5, 1), 0.67);
    assert_eq!(logic::step_zoom(0.5, -1), 0.33);
    assert_eq!(logic::step_zoom(0.47, 1), 0.5);
    assert_eq!(logic::step_zoom(4.0, 1), 4.0);
    assert_eq!(logic::step_zoom(0.1, -1), 0.1);
    assert_eq!(logic::clamp_zoom(9.0), 4.0);
    assert_eq!(logic::clamp_zoom(0.0), 0.1);
}

#[test]
fn text_is_clipped_by_characters_and_a_note_stays_on_one_line() {
    assert_eq!(logic::clip("a\nb\r\nc", 10, false), "a b c");
    assert_eq!(logic::clip("a \n  b", 10, false), "a b");
    assert_eq!(logic::clip("a  b", 10, false), "a  b");
    assert_eq!(logic::clip("a\nb", 10, true), "a\nb");
    assert_eq!(logic::clip("éééé", 2, false), "éé");
    assert_eq!(logic::clip(&"😀".repeat(10), 3, false).chars().count(), 3);
}

#[test]
fn the_export_scales_go_down_and_the_message_limit_is_panes_send_s() {
    let mut sorted = logic::EXPORT_SCALES;
    sorted.sort_by(|a, b| b.total_cmp(a));
    assert_eq!(sorted, logic::EXPORT_SCALES);
    assert_eq!(logic::EXPORT_SCALES[0], 1.0);
    assert_eq!(answer::MESSAGE_MAX, 20_000);
    assert_eq!(model::Color::ALL.len(), 5);
}
