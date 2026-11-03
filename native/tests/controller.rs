//! test/controller.test.js, ported: what opens, what is sent, what is said
//! when something is refused, on the harness, Hangar's answers given by hand.

mod common;

use serde_json::{Value, json};

use hangar_kit_sdk::testing::{Harness, StartExt};
use hangar_kit_sdk::ui::abi::Operation;
use hangar_kit_sdk::ui::{ImageSource, Mark, Point};

use common::*;
use kit_annotate_tab::model::{Color, Shape, Tool};
use kit_annotate_tab::{AlertKind, StatusKind};

fn arrow() -> Mark {
    Mark::arrow((0.0, 0.0), (100.0, 100.0))
}

fn empty_tab() -> Harness<Annotate> {
    Harness::new(start())
}

#[test]
fn a_path_param_opens_that_image_from_the_workspace_and_nothing_else_is_read() {
    let mut tab = asking(".hangar/capture.png", start());
    assert_eq!(tab.kit().phase, Phase::Loading);
    assert!(tab.shows("Reading .hangar/capture.png…"));
    let call = tab.reply_to("image.size", Ok(json!({ "width": 800, "height": 600 })));
    assert_eq!(
        call.params,
        json!({ "source": { "file": ".hangar/capture.png" } })
    );
    let state = tab.kit();
    assert_eq!(state.phase, Phase::Ready);
    let image = state.image.as_ref().unwrap();
    assert_eq!(
        image.source,
        ImageSource::File(".hangar/capture.png".into())
    );
    assert_eq!(
        (image.width, image.height, image.name.as_str()),
        (800, 600, "capture.png")
    );
    assert_eq!(state.source.as_deref(), Some(".hangar/capture.png"));
    assert!(
        tab.calls().is_empty(),
        "the capture itself is never read into the tab"
    );
    assert_eq!(
        canvas(&tab).unwrap().background,
        Some(ImageSource::File(".hangar/capture.png".into()))
    );
}

#[test]
fn with_no_path_the_tab_waits_for_a_pasted_or_chosen_image() {
    let mut tab = empty_tab();
    assert_eq!(tab.kit().phase, Phase::Empty);
    assert!(tab.calls().is_empty());
    tab.shortcut("mod+v");
    tab.reply_to(
        "ui.pasteImage",
        Ok(json!({ "held": "held-1", "width": 800, "height": 600 })),
    );
    let state = tab.kit();
    assert_eq!(state.phase, Phase::Ready);
    assert_eq!(state.source, None);
    assert_eq!(
        state.image.as_ref().unwrap().source,
        ImageSource::Held("held-1".into())
    );

    let mut chosen = empty_tab();
    chosen.press("Choose an image");
    let pick = chosen.reply_to(
        "files.pick",
        Ok(json!({ "name": "pasted.png", "size": 1000, "held": "held-2" })),
    );
    assert_eq!(
        pick.params["extensions"],
        json!(["png", "jpg", "jpeg", "webp", "gif"])
    );
    let size = chosen.reply_to("image.size", Ok(json!({ "width": 800, "height": 600 })));
    assert_eq!(size.params, json!({ "source": { "held": "held-2" } }));
    assert_eq!(chosen.kit().image.as_ref().unwrap().name, "pasted.png");
    assert_eq!(chosen.kit().source, None);

    // A file of the workspace, chosen, is shown where it is.
    let mut inside = empty_tab();
    inside.press("Choose an image");
    inside.reply_to(
        "files.pick",
        Ok(json!({ "name": "b.png", "size": 1000, "path": "shots/b.png" })),
    );
    let size = inside.reply_to("image.size", Ok(json!({ "width": 10, "height": 20 })));
    assert_eq!(size.params, json!({ "source": { "file": "shots/b.png" } }));
    assert_eq!(inside.kit().source.as_deref(), Some("shots/b.png"));

    // Cancelling the chooser changes nothing.
    let mut cancelled = empty_tab();
    cancelled.press("Choose an image");
    cancelled.reply_to("files.pick", Ok(Value::Null));
    assert_eq!(cancelled.kit().phase, Phase::Empty);
    assert!(cancelled.calls().is_empty());
}

#[test]
fn what_is_not_a_bitmap_is_refused_in_words_and_nothing_opens() {
    let mut tab = empty_tab();
    tab.press("Choose an image");
    tab.reply_to(
        "files.pick",
        Ok(json!({ "name": "x.svg", "size": 10, "path": "x.svg" })),
    );
    tab.reply_to("image.size", refusal("unsupported"));
    assert_eq!(tab.kit().phase, Phase::Empty);
    assert!(
        tab.kit()
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("not an image")
    );

    tab.press("Choose an image");
    tab.reply_to(
        "files.pick",
        Ok(json!({ "name": "huge.png", "size": 30 * 1024 * 1024 + 1, "path": "huge.png" })),
    );
    assert!(tab.kit().notice.as_ref().unwrap().text.contains("30 MB"));
    assert!(tab.calls().is_empty());

    tab.shortcut("mod+v");
    tab.reply_to("ui.pasteImage", refusal("not_found"));
    assert!(
        tab.kit()
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("clipboard holds no image")
    );
    assert_eq!(tab.kit().phase, Phase::Empty);
}

#[test]
fn a_path_that_is_not_found_not_an_image_or_an_svg_says_so_and_offers_another_way_in() {
    let mut missing = asking("nope.png", start());
    missing.reply_to("image.size", refusal("not_found"));
    assert_eq!(missing.kit().phase, Phase::Empty);
    assert!(missing.shows("nope.png was not found"));
    assert!(missing.shows("Choose an image"));

    let mut svg = asking("a.svg", start());
    svg.reply_to("image.size", refusal("unsupported"));
    assert_eq!(svg.kit().phase, Phase::Empty);
    assert!(svg.shows("not a PNG, JPEG, WebP or GIF"));

    let mut broken = asking("a.png", start());
    broken.reply_to("image.size", refusal("failed"));
    assert_eq!(
        broken.kit().notice.as_ref().unwrap().retry.as_deref(),
        Some("a.png")
    );
    broken.press("Try again");
    assert_eq!(broken.methods(), ["image.size"]);
    broken.reply_to("image.size", Ok(json!({ "width": 4, "height": 3 })));
    assert_eq!(broken.kit().phase, Phase::Ready);
}

#[test]
fn denied_reading_says_what_to_accept_and_pasting_still_works() {
    let mut tab = asking("a.png", start());
    tab.reply_to("image.size", refusal("denied"));
    let notice = tab.kit().notice.clone().unwrap();
    assert_eq!(notice.kind, AlertKind::Denied);
    assert!(notice.text.contains("files.read"));
    assert!(notice.text.contains("paste"));
    tab.press("Paste an image");
    tab.reply_to(
        "ui.pasteImage",
        Ok(json!({ "held": "held-1", "width": 800, "height": 600 })),
    );
    assert_eq!(tab.kit().phase, Phase::Ready);

    // The chooser needs files.read too: refused, it says the same.
    let mut picking = empty_tab();
    picking.press("Choose an image");
    picking.reply_to("files.pick", refusal("denied"));
    assert_eq!(
        picking.kit().notice.as_ref().unwrap().kind,
        AlertKind::Denied
    );
}

#[test]
fn the_tab_starts_from_what_hangar_hands_it_and_asks_nothing_first() {
    // The page asked `tab.params`; a native tab is handed its params and
    // caller in its start.
    let tab = empty_tab();
    assert!(tab.calls().is_empty());
    assert_eq!(tab.kit().caller, None);
    assert_eq!(tab.kit().phase, Phase::Empty);
    let called = opened_with("a.png", start().caller("Ava"));
    assert_eq!(called.kit().caller.as_deref(), Some("Ava"));
    assert_eq!(called.kit().target, "Ava");
}

#[test]
fn tools_colours_and_zoom_change_only_to_what_exists() {
    let mut tab = opened("a.png");
    tab.press("Rectangle");
    assert_eq!(tab.kit().tool, Tool::Rect);
    let tools: Vec<String> = filters(&tab)
        .into_iter()
        .map(|filter| filter.label)
        .collect();
    assert_eq!(
        tools,
        ["Arrow", "Rectangle", "Freehand", "Note"],
        "no laser"
    );
    tab.press("Blue");
    assert_eq!(tab.kit().color, Color::Blue);
    for _ in 0..20 {
        tab.press("Zoom in");
    }
    assert_eq!(tab.kit().zoom, Some(4.0));
    for _ in 0..20 {
        tab.press("Zoom out");
    }
    assert_eq!(tab.kit().zoom, Some(0.1));
    tab.press("Fit");
    assert_eq!(tab.kit().zoom, None);
}

#[test]
fn marks_are_added_notes_are_numbered_typing_changes_the_word_and_undo_takes_the_last() {
    let mut tab = opened("a.png");
    tab.draw("shot", arrow());
    tab.press("Green");
    tab.press("Note");
    tab.take_operations();
    tab.point("shot", Point::new(10.0, 20.0));
    assert_eq!(
        tab.take_operations(),
        [Operation::Focus {
            id: "note-1".into()
        }],
        "the note's field takes the focus once"
    );
    type_into_id(&mut tab, "note-1", "too close\nto the edge");
    let Shape::Note { text, color, .. } = &tab.kit().shapes[1] else {
        panic!("a note");
    };
    assert_eq!(text, "too close to the edge", "one line");
    assert_eq!(*color, Color::Green);
    tab.point("shot", Point::new(30.0, 40.0));
    assert_eq!(
        tab.take_operations(),
        [Operation::Focus {
            id: "note-2".into()
        }]
    );
    assert_eq!(tab.kit().shapes.len(), 3);

    // Ctrl+Z in a note's field is the field's: no mark goes.
    tab.shortcut("mod+z");
    assert_eq!(tab.kit().shapes.len(), 3);
    tab.press("Undo");
    assert_eq!(tab.kit().shapes.len(), 2);
    tab.shortcut("mod+z");
    assert_eq!(
        tab.kit().shapes.len(),
        1,
        "once a press left the field, Ctrl+Z undoes"
    );
    type_into_id(&mut tab, "target", "Jack");
    tab.shortcut("mod+z");
    assert_eq!(
        tab.kit().shapes.len(),
        1,
        "typing the pane's name is a field too"
    );
    tab.press("Arrow");
    // A press on the canvas leaves the field too; with the arrow it draws
    // nothing.
    tab.point("shot", Point::new(1.0, 1.0));
    assert_eq!(tab.kit().shapes.len(), 1);
    tab.shortcut("mod+z");
    assert_eq!(tab.kit().shapes.len(), 0);
    tab.shortcut("mod+z");
    assert_eq!(tab.kit().shapes.len(), 0);
    assert!(
        button(&tab, "Undo").unwrap().on_press.is_none(),
        "nothing to undo"
    );
}

#[test]
fn a_marked_image_is_not_replaced_by_a_paste_and_another_image_drops_it() {
    let mut tab = opened("a.png");
    tab.draw("shot", arrow());
    tab.shortcut("mod+v");
    assert!(tab.calls().is_empty(), "the clipboard is not read");
    assert_eq!(tab.kit().image.as_ref().unwrap().name, "a.png");
    assert!(
        tab.kit()
            .status
            .as_ref()
            .unwrap()
            .text
            .contains("Another image")
    );
    tab.press("Another image");
    tab.reply_to("ui.confirm", Ok(json!(true)));
    assert_eq!(tab.kit().phase, Phase::Empty);
    assert!(tab.kit().shapes.is_empty());
    tab.shortcut("mod+v");
    tab.reply_to(
        "ui.pasteImage",
        Ok(json!({ "held": "held-3", "width": 8, "height": 6 })),
    );
    assert_eq!(
        tab.kit().image.as_ref().unwrap().source,
        ImageSource::Held("held-3".into())
    );
}

#[test]
fn send_writes_the_png_then_the_notes_then_tells_the_agent_that_asked() {
    let mut tab = opened_with(".hangar/capture.png", start().caller("Ava"));
    assert_eq!(tab.kit().target, "Ava");
    tab.draw("shot", arrow());
    tab.press("Note");
    tab.point("shot", Point::new(120.0, 240.0));
    type_into_id(&mut tab, "note-1", "too close");
    tab.type_into("Comment (optional)", "Fix the spacing.");
    tab.press("Send");

    let export = tab.reply_to("canvas.export", exported(PNG, 1.0));
    assert_eq!(export.params["path"], PNG);
    assert_eq!(
        (
            export.params["width"].clone(),
            export.params["height"].clone()
        ),
        (json!(800.0), json!(600.0))
    );
    assert_eq!(
        export.params["background"],
        json!({ "file": ".hangar/capture.png" })
    );
    assert_eq!(
        export.params["marks"].as_array().unwrap().len(),
        4,
        "an arrow, a badge, its number, its word"
    );

    let write = tab.reply_to("files.write", Ok(json!({})));
    assert_eq!(write.params["path"], JSON);
    let record: Value = serde_json::from_str(write.params["text"].as_str().unwrap()).unwrap();
    assert_eq!(record["image"], "20261031-231512.png");
    assert_eq!(
        [
            &record["width"],
            &record["height"],
            &record["source"],
            &record["comment"]
        ],
        [
            &json!(800),
            &json!(600),
            &json!(".hangar/capture.png"),
            &json!("Fix the spacing.")
        ]
    );
    assert_eq!(
        record["notes"],
        json!([{ "n": 1, "x": 120, "y": 240, "text": "too close" }])
    );
    assert_eq!(record["marks"].as_array().unwrap().len(), 2);
    assert_eq!(record["createdAt"], "2026-10-31T22:15:12.000Z");

    let sent = tab.reply_to("panes.send", Ok(json!({ "queued": true, "pane": "Ava" })));
    assert_eq!(sent.params["pane"], "Ava");
    let text = sent.params["text"].as_str().unwrap();
    assert!(text.contains(&format!("Image: {PNG}")));
    assert!(text.contains("Comment: Fix the spacing."));
    assert!(text.contains("1. too close (x 120, y 240)"));
    let status = tab.kit().status.clone().unwrap();
    assert_eq!(status.kind, StatusKind::Success);
    assert_eq!(status.text, format!("Sent to Ava: {PNG}"));
    assert!(!tab.kit().busy);
    assert!(tab.calls().is_empty());
}

#[test]
fn the_pane_the_person_types_is_the_one_told_and_without_one_the_files_are_only_saved() {
    let mut typed = opened("a.png");
    typed.type_into("Pane name", "  Jack ");
    typed.press("Send");
    typed.reply_to("canvas.export", exported(PNG, 1.0));
    typed.reply_to("files.write", Ok(json!({})));
    assert_eq!(
        typed
            .reply_to("panes.send", Ok(json!({ "queued": true, "pane": "Jack" })))
            .params["pane"],
        "Jack"
    );

    let mut alone = opened("a.png");
    alone.press("Save");
    alone.reply_to("canvas.export", exported(PNG, 1.0));
    alone.reply_to("files.write", Ok(json!({})));
    assert!(alone.calls().is_empty(), "no pane is told");
    assert_eq!(
        alone.kit().status.as_ref().unwrap().text,
        format!("Saved: {PNG}")
    );
}

#[test]
fn a_taken_name_gets_a_counter_instead_of_replacing_anything() {
    let mut tab = opened("a.png");
    tab.press("Save");
    tab.reply_to("canvas.export", refusal("conflict"));
    let second = tab.reply_to(
        "canvas.export",
        exported(".hangar/annotations/20261031-231512-2.png", 1.0),
    );
    assert_eq!(
        second.params["path"],
        ".hangar/annotations/20261031-231512-2.png"
    );
    let write = tab.reply_to("files.write", Ok(json!({})));
    assert_eq!(
        write.params["path"],
        ".hangar/annotations/20261031-231512-2.json"
    );
    assert!(tab.kit().status.as_ref().unwrap().text.ends_with("-2.png"));

    // Five names taken: nothing is replaced, and the person is told.
    let mut full = opened("a.png");
    full.press("Save");
    for _ in 0..5 {
        full.reply_to("canvas.export", refusal("conflict"));
    }
    assert!(full.calls().is_empty());
    assert!(
        full.kit()
            .status
            .as_ref()
            .unwrap()
            .text
            .contains("name taken")
    );
}

#[test]
fn a_heavy_image_is_written_at_a_smaller_scale_and_the_notes_follow_the_scale() {
    let mut tab = opened("a.png");
    tab.type_into("Pane name", "Ava");
    tab.press("Note");
    tab.point("shot", Point::new(400.0, 200.0));
    tab.press("Send");
    tab.reply_to("canvas.export", exported(PNG, 0.5));
    let write = tab.reply_to("files.write", Ok(json!({})));
    let record: Value = serde_json::from_str(write.params["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        [&record["width"], &record["height"]],
        [&json!(400), &json!(300)]
    );
    assert_eq!(
        [&record["notes"][0]["x"], &record["notes"][0]["y"]],
        [&json!(200), &json!(100)]
    );
    assert_eq!(
        record["marks"][0],
        json!({ "type": "note", "color": "red", "x": 200, "y": 100 })
    );
    let sent = tab.reply_to("panes.send", Ok(json!({ "queued": true, "pane": "Ava" })));
    assert!(
        sent.params["text"]
            .as_str()
            .unwrap()
            .contains("400 × 300 px, reduced from the original")
    );
    assert_eq!(tab.kit().status.as_ref().unwrap().kind, StatusKind::Success);

    let mut never = opened("a.png");
    never.press("Save");
    never.reply_to(
        "canvas.export",
        Err(hangar_kit_sdk::KitError::new(
            "failed",
            "the picture is larger than 10 MB even at a quarter of its size",
        )),
    );
    assert!(never.calls().is_empty(), "no notes file without its image");
    assert!(
        never
            .kit()
            .status
            .as_ref()
            .unwrap()
            .text
            .contains("too heavy")
    );
}

#[test]
fn denied_writing_says_what_to_accept_and_nothing_is_sent() {
    let mut tab = opened_with("a.png", start().caller("Ava"));
    tab.press("Send");
    tab.reply_to("canvas.export", refusal("denied"));
    let status = tab.kit().status.clone().unwrap();
    assert_eq!(status.kind, StatusKind::Error);
    assert!(status.text.contains("files.write in Settings › Kits"));
    assert!(tab.calls().is_empty(), "no pane is told");
    assert!(!tab.kit().busy);
}

#[test]
fn a_hangar_without_canvas_export_is_told_to_update_in_french_too() {
    let mut tab = opened_with("a.png", start().locale("fr"));
    tab.press("Enregistrer");
    tab.reply_to("canvas.export", refusal("unknown"));
    assert!(
        tab.kit()
            .status
            .as_ref()
            .unwrap()
            .text
            .contains("Mettez Hangar à jour")
    );
}

#[test]
fn when_the_agent_cannot_be_told_the_image_is_still_saved_and_the_person_is_told_what_to_do() {
    let cases = [
        ("denied", "panes.send in Settings › Kits"),
        ("not_found", "No pane is named Ava"),
        ("unavailable", "Ava cannot take a message"),
        ("rate_limited", "Ava was not told"),
    ];
    for (code, said) in cases {
        let mut tab = opened_with("a.png", start().caller("Ava"));
        tab.press("Send");
        tab.reply_to("canvas.export", exported(PNG, 1.0));
        tab.reply_to("files.write", Ok(json!({})));
        tab.reply_to("panes.send", refusal(code));
        let status = tab.kit().status.clone().unwrap();
        assert_eq!(status.kind, StatusKind::Warning, "{code}");
        assert!(
            status.text.starts_with(&format!("Saved: {PNG}")),
            "{code}: the image is saved"
        );
        assert!(status.text.contains(said), "{code}: {}", status.text);
    }
}

#[test]
fn a_notes_file_that_cannot_be_saved_does_not_lose_the_image_or_the_message() {
    let mut tab = opened_with("a.png", start().caller("Ava"));
    tab.press("Send");
    tab.reply_to("canvas.export", exported(PNG, 1.0));
    tab.reply_to("files.write", refusal("failed"));
    let sent = tab.reply_to("panes.send", Ok(json!({ "queued": true, "pane": "Ava" })));
    assert!(
        !sent.params["text"]
            .as_str()
            .unwrap()
            .contains("Notes file:")
    );
    let status = tab.kit().status.clone().unwrap();
    assert_eq!(status.kind, StatusKind::Warning);
    assert!(status.text.contains("not its notes file"));
}

#[test]
fn a_second_send_while_one_runs_does_nothing_and_there_is_nothing_to_send_without_an_image() {
    let empty = empty_tab();
    assert!(button(&empty, "Save").is_none() && button(&empty, "Send").is_none());

    let mut tab = opened("a.png");
    let send = button(&tab, "Save").unwrap().on_press.unwrap();
    let seq = tab.view().seq;
    tab.press("Save");
    assert!(tab.shows("Sending…"));
    assert!(
        button(&tab, "Sending…").unwrap().on_press.is_none(),
        "disabled while it runs"
    );
    // A second click, drawn before the first was seen, still reaches Send.
    tab.ui_from(seq, send, None);
    assert_eq!(tab.methods(), ["canvas.export"]);
}

#[test]
fn asked_for_again_the_latest_agent_is_the_one_to_answer_and_marks_are_never_lost() {
    let mut tab = opened_with("a.png", start().caller("Ava"));
    tab.draw("shot", arrow());
    let again = json!({ "params": { "path": "b.png" }, "caller": { "pane": "Jack" } });
    tab.event("tab.params", again.clone());
    assert_eq!(tab.kit().target, "Jack");
    assert_eq!(
        tab.kit().source.as_deref(),
        Some("a.png"),
        "marks stay on their image"
    );
    assert!(tab.calls().is_empty());
    tab.press("Undo");
    tab.event("tab.params", again.clone());
    tab.reply_to("image.size", Ok(json!({ "width": 800, "height": 600 })));
    assert_eq!(tab.kit().source.as_deref(), Some("b.png"));
    tab.event("tab.params", again);
    assert!(tab.calls().is_empty(), "the same image is not read again");
}

#[test]
fn an_image_that_arrives_late_does_not_replace_a_newer_one() {
    let mut tab = asking("a.png", start());
    tab.shortcut("mod+v");
    tab.reply_to(
        "ui.pasteImage",
        Ok(json!({ "held": "held-1", "width": 8, "height": 6 })),
    );
    tab.reply_to("image.size", Ok(json!({ "width": 800, "height": 600 })));
    let image = tab.kit().image.clone().unwrap();
    assert_eq!(image.source, ImageSource::Held("held-1".into()));
    assert_eq!(tab.kit().source, None);
}

#[test]
fn a_paste_that_fails_while_an_image_is_looked_for_does_not_leave_the_tab_loading() {
    let mut tab = asking("a.png", start());
    tab.shortcut("mod+v");
    tab.reply_to("ui.pasteImage", refusal("not_found"));
    tab.reply_to("image.size", Ok(json!({ "width": 800, "height": 600 })));
    assert_eq!(
        tab.kit().phase,
        Phase::Empty,
        "the newer ask won, and failed"
    );
    assert!(tab.shows("The clipboard holds no image."));
    assert!(tab.shows("Choose an image"));
}
