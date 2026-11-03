//! What the tab draws for each state, in both
//! languages, the canvas driven as Hangar drives it.

mod common;

use serde_json::json;

use hangar_kit_sdk::testing::{Harness, StartExt};
use hangar_kit_sdk::ui::abi::Operation;
use hangar_kit_sdk::ui::{
    ButtonKind, Element, EmptyKind, ImageSource, Ink, Mark, Notice, Point, Tone, Tool as Pen,
};

use common::*;
use kit_annotate_tab::model::{Color, Shape, Tool};

#[test]
fn with_no_image_an_empty_state_that_says_how_to_open_one_and_a_button_that_picks_a_file() {
    let mut tab = Harness::<Annotate>::new(start());
    let empty = &empties(&tab)[0];
    assert_eq!(empty.kind, EmptyKind::Empty);
    assert_eq!(empty.title, "No image yet");
    assert!(empty.text.as_deref().unwrap().contains("Ctrl+V"));
    assert!(canvas(&tab).is_none());
    assert!(button(&tab, "Save").is_none(), "nothing to send yet");
    let primary: Vec<_> = buttons(&tab)
        .into_iter()
        .filter(|b| b.kind == ButtonKind::Primary)
        .collect();
    assert_eq!(primary.len(), 1);
    assert_eq!(primary[0].label.as_deref(), Some("Choose an image"));
    tab.press("Choose an image");
    assert_eq!(tab.methods(), ["files.pick"]);
}

#[test]
fn in_french_with_accents() {
    let tab = opened_with("a.png", start().locale("fr"));
    assert!(tab.shows("Annoter"));
    for label in [
        "Flèche",
        "Rectangle",
        "Main levée",
        "Note",
        "Annuler",
        "Ajuster",
        "Autre image",
    ] {
        assert!(tab.shows(label), "{label}");
    }
    assert!(
        button(&tab, "Enregistrer").is_some(),
        "no pane to send to yet"
    );
    assert!(says(&tab, "Commentaire (facultatif)"));
    assert!(says(&tab, "Sans nom de pane"));
    assert!(says(&tab, "Rouge") && says(&tab, "Vert"));
}

#[test]
fn the_language_follows_hangar_s() {
    let mut tab = opened("a.png");
    assert!(tab.shows("Annotate"));
    tab.event("theme", json!({ "scheme": "light", "locale": "fr-FR" }));
    assert!(tab.shows("Annoter"));
    assert!(tab.shows("Flèche"));
}

#[test]
fn with_an_image_tools_as_toggles_five_colours_undo_zoom_the_canvas_and_one_primary_action() {
    let tab = opened("a.png");
    let chips = filters(&tab);
    let labels: Vec<&str> = chips.iter().map(|chip| chip.label.as_str()).collect();
    assert_eq!(labels, ["Arrow", "Rectangle", "Freehand", "Note"]);
    let pressed: Vec<bool> = chips.iter().map(|chip| chip.pressed).collect();
    assert_eq!(pressed, [true, false, false, false]);
    let colours = swatches(&tab);
    let names: Vec<&str> = colours.iter().map(|swatch| swatch.label.as_str()).collect();
    assert_eq!(names, ["Red", "Orange", "Green", "Blue", "Magenta"]);
    let inks: Vec<Ink> = colours.iter().map(|swatch| swatch.ink).collect();
    assert_eq!(inks, Color::ALL.map(Color::ink));
    assert!(colours[0].selected && colours[1..].iter().all(|swatch| !swatch.selected));
    assert!(
        button(&tab, "Undo").unwrap().on_press.is_none(),
        "nothing to undo"
    );
    assert!(button(&tab, "Fit").is_some());
    let primary: Vec<_> = buttons(&tab)
        .into_iter()
        .filter(|b| b.kind == ButtonKind::Primary)
        .collect();
    assert_eq!(primary.len(), 1);
    assert_eq!(primary[0].label.as_deref(), Some("Save"));
    assert!(tab.shows("a.png · 800 × 600 px"));
    // Fitted to the panel by Hangar, in the image's own pixels.
    let board = canvas(&tab).unwrap();
    assert_eq!(
        (board.id.as_str(), board.width, board.height),
        ("shot", 800.0, 600.0)
    );
    assert_eq!(board.zoom, None);
    assert!(board.on_fit.is_some() && board.on_draw.is_some() && board.on_point.is_some());
    assert_eq!(board.description.as_deref(), Some("The image to annotate"));
}

#[test]
fn choosing_a_tool_or_a_colour_redraws_with_it_pressed_and_the_tip_follows_the_tool() {
    let mut tab = opened("a.png");
    tab.press("Rectangle");
    assert_eq!(tab.kit().tool, Tool::Rect);
    assert!(
        filters(&tab)
            .iter()
            .any(|chip| chip.label == "Rectangle" && chip.pressed)
    );
    assert!(tab.shows("Drag on the image to frame something."));
    assert_eq!(
        canvas(&tab).unwrap().tool,
        Some(Pen::Rect),
        "what a drag draws"
    );
    tab.press("Blue");
    assert_eq!(tab.kit().color, Color::Blue);
    assert!(swatches(&tab)[3].selected);
    assert_eq!(canvas(&tab).unwrap().ink, Ink::Blue);
}

#[test]
fn dragging_with_the_arrow_tool_keeps_the_mark_a_speck_keeps_nothing() {
    let mut tab = opened("a.png");
    assert_eq!(canvas(&tab).unwrap().tool, Some(Pen::Arrow));
    tab.draw(
        "shot",
        Mark::arrow((10.0, 10.0), (210.0, 130.0)).ink(Ink::Red),
    );
    assert_eq!(
        tab.kit().shapes,
        [Shape::Arrow {
            from: Point::new(10.0, 10.0),
            to: Point::new(210.0, 130.0),
            color: Color::Red
        }]
    );
    assert_eq!(
        canvas(&tab).unwrap().marks,
        [Mark::arrow((10.0, 10.0), (210.0, 130.0)).ink(Ink::Red)]
    );
    tab.draw("shot", Mark::arrow((50.0, 50.0), (51.0, 51.0)));
    assert_eq!(tab.kit().shapes.len(), 1, "too small to mean anything");
    assert!(button(&tab, "Undo").unwrap().on_press.is_some());
}

#[test]
fn the_freehand_tool_keeps_the_stroke() {
    let mut tab = opened("a.png");
    tab.press("Freehand");
    assert_eq!(canvas(&tab).unwrap().tool, Some(Pen::Free));
    let points = [
        (0.0, 0.0),
        (20.0, 5.0),
        (40.0, 30.0),
        (60.0, 10.0),
        (80.0, 40.0),
    ]
    .map(|(x, y)| Point::new(x, y));
    tab.draw("shot", Mark::path(points));
    let Shape::Free { points, .. } = &tab.kit().shapes[0] else {
        panic!("a stroke");
    };
    assert!(points.len() >= 4);
}

#[test]
fn a_click_with_the_note_tool_places_a_numbered_note_and_its_word_is_typed_in_the_list() {
    let mut tab = opened("a.png");
    assert!(tab.shows("Notes you place on the image are listed here."));
    tab.press("Note");
    assert_eq!(
        canvas(&tab).unwrap().tool,
        Some(Pen::Point),
        "a press is a point"
    );
    tab.take_operations();
    tab.point("shot", Point::new(300.0, 200.0));
    let Shape::Note { at, .. } = &tab.kit().shapes[0] else {
        panic!("a note");
    };
    assert_eq!(*at, Point::new(300.0, 200.0));
    let field = input(&tab, "note-1").expect("a field for the note");
    assert_eq!(field.placeholder, "What is wrong here?");
    assert_eq!(
        tab.take_operations(),
        [Operation::Focus {
            id: "note-1".into()
        }],
        "it takes the focus, because the person just clicked"
    );
    type_into_id(&mut tab, "note-1", "<b>too close</b>");
    let Shape::Note { text, .. } = &tab.kit().shapes[0] else {
        panic!("a note");
    };
    assert_eq!(text, "<b>too close</b>");
    assert!(
        tab.shows("<b>too close</b>"),
        "drawn with its word, as text"
    );
    assert_eq!(input(&tab, "note-1").unwrap().value, "<b>too close</b>");
    assert!(
        tab.take_operations().is_empty(),
        "typing does not move the focus"
    );
}

#[test]
fn the_send_button_says_send_once_there_is_a_pane_to_tell_and_sending_while_it_works() {
    let mut tab = opened_with("a.png", start().caller("Ava"));
    let primary = || ButtonKind::Primary;
    let label = |tab: &Harness<Annotate>| {
        buttons(tab)
            .into_iter()
            .find(|button| button.kind == primary())
            .and_then(|button| button.label)
            .unwrap()
    };
    assert_eq!(label(&tab), "Send");
    assert!(says(&tab, "Ava asked for this image."));
    tab.type_into("Pane name", "");
    assert_eq!(label(&tab), "Save");
    assert!(says(&tab, "Without a pane name, the files are only saved."));
    tab.type_into("Pane name", "Jack");
    assert_eq!(label(&tab), "Send");
    assert!(!says(&tab, "Without a pane name"));
    tab.press("Send");
    assert_eq!(label(&tab), "Sending…");
    assert!(tab.kit().busy);
}

fn foot_tone(tab: &Harness<Annotate>, said: &str) -> Option<Tone> {
    widgets(tab).into_iter().find_map(|element| match element {
        Element::Text(text) if text.text == said => Some(text.tone),
        _ => None,
    })
}

#[test]
fn the_sent_answer_shows_in_the_foot_an_error_in_the_danger_tone() {
    let mut tab = opened_with("a.png", start().caller("Ava"));
    tab.press("Send");
    tab.reply_to("canvas.export", exported(PNG, 1.0));
    tab.reply_to("files.write", Ok(json!({})));
    tab.reply_to("panes.send", Ok(json!({ "queued": true, "pane": "Ava" })));
    let said = format!("Sent to Ava: {PNG}");
    assert_eq!(foot_tone(&tab, &said), Some(Tone::Default));

    let mut denied = opened("a.png");
    denied.press("Save");
    denied.reply_to("canvas.export", refusal("denied"));
    let said = denied.kit().status.clone().unwrap().text;
    assert_eq!(foot_tone(&denied, &said), Some(Tone::Danger));
}

#[test]
fn denied_reading_is_a_designed_state_with_a_way_to_paste_anyway() {
    let mut tab = asking("a.png", start());
    tab.reply_to("image.size", refusal("denied"));
    let empty = &empties(&tab)[0];
    assert_eq!(empty.kind, EmptyKind::Denied);
    assert_eq!(empty.title, "This tab may not read files");
    assert_eq!(
        empties(&tab).len(),
        1,
        "the denied state, not the empty one too"
    );
    tab.press("Paste an image");
    assert_eq!(tab.methods(), ["ui.pasteImage"]);
}

#[test]
fn a_path_not_found_is_an_error_banner_with_the_empty_state_below_it() {
    let mut tab = asking("nope.png", start());
    assert_eq!(
        empties(&tab)[0].kind,
        EmptyKind::Loading,
        "while it is looked for"
    );
    tab.reply_to("image.size", refusal("not_found"));
    let banner = &banners(&tab)[0];
    assert_eq!(banner.kind, Notice::Error);
    assert!(banner.text.contains("nope.png was not found"));
    assert_eq!(empties(&tab)[0].kind, EmptyKind::Empty);
}

#[test]
fn another_image_asks_before_dropping_marks_and_not_otherwise() {
    let mut bare = opened("a.png");
    bare.press("Another image");
    assert_eq!(bare.kit().phase, Phase::Empty);
    assert!(bare.calls().is_empty(), "nothing to ask");

    let mut marked = opened("a.png");
    marked.draw("shot", Mark::arrow((0.0, 0.0), (90.0, 90.0)));
    marked.press("Another image");
    let asked = marked.reply_to("ui.confirm", Ok(json!(false)));
    assert_eq!(
        asked.params["message"],
        "Replace this image and drop its marks?"
    );
    assert_eq!(asked.params["confirm"], "Replace");
    assert_eq!(asked.params["cancel"], "Keep");
    assert_eq!(asked.params["danger"], true);
    assert_eq!(marked.kit().phase, Phase::Ready, "kept");

    let mut yes = opened("a.png");
    yes.draw("shot", Mark::arrow((0.0, 0.0), (90.0, 90.0)));
    yes.press("Another image");
    yes.reply_to("ui.confirm", Ok(json!(true)));
    assert_eq!(yes.kit().phase, Phase::Empty);
}

#[test]
fn zoom_steps_from_the_fitted_zoom_and_fit_goes_back() {
    let mut tab = opened("a.png");
    fit(&mut tab, 1.0);
    assert!(tab.shows("100 %"));
    tab.press("Zoom in");
    assert_eq!(tab.kit().zoom, Some(1.5));
    assert!(tab.shows("150 %"));
    let board = canvas(&tab).unwrap();
    assert_eq!(board.zoom, Some(1.5));
    assert!(board.on_fit.is_none());
    let scrolled = widgets(&tab).into_iter().any(|element| {
        matches!(element, Element::Scroll(scroll) if matches!(*scroll.child, Element::Canvas(_)))
    });
    assert!(scrolled, "past the panel, the canvas scrolls both ways");
    tab.press("Zoom out");
    assert_eq!(tab.kit().zoom, Some(1.0));
    tab.press("Fit");
    assert_eq!(tab.kit().zoom, None);

    // Fitted at 47 %, the next step up is 50 %.
    fit(&mut tab, 0.47);
    assert!(tab.shows("47 %"));
    tab.press("Zoom in");
    assert_eq!(tab.kit().zoom, Some(0.5));
}

#[test]
fn every_control_is_named() {
    let tab = opened("a.png");
    let buttons = buttons(&tab);
    for button in &buttons {
        assert!(
            button
                .label
                .as_deref()
                .or(button.tooltip.as_deref())
                .is_some_and(|name| !name.is_empty()),
            "a button with no name: {button:?}"
        );
    }
    let controls =
        buttons.len() + filters(&tab).len() + swatches(&tab).len() + inputs(&tab).len() + 1;
    assert!(controls >= 15, "{controls}");
    for input in inputs(&tab) {
        assert!(!input.placeholder.is_empty() || input.label.is_some());
    }
    assert!(swatches(&tab).iter().all(|swatch| !swatch.label.is_empty()));
}

#[test]
fn a_chosen_image_shows_its_own_name_as_text_and_a_pasted_one_its_size() {
    let mut tab = Harness::<Annotate>::new(start());
    tab.press("Choose an image");
    tab.reply_to(
        "files.pick",
        Ok(json!({ "name": "shot <1>.png", "size": 10, "held": "held-1" })),
    );
    tab.reply_to("image.size", Ok(json!({ "width": 800, "height": 600 })));
    assert!(tab.shows("shot <1>.png · 800 × 600 px"));
    assert_eq!(
        canvas(&tab).unwrap().background,
        Some(ImageSource::Held("held-1".into()))
    );

    let mut pasted = Harness::<Annotate>::new(start());
    pasted.press("Paste an image");
    pasted.reply_to(
        "ui.pasteImage",
        Ok(json!({ "held": "held-2", "width": 640, "height": 400 })),
    );
    assert!(pasted.shows("640 × 400 px"));
    assert!(!pasted.shows("· 640"));
}

#[test]
fn the_page_is_a_header_a_part_that_scrolls_and_a_foot() {
    let tab = opened("a.png");
    let Element::Page(page) = &tab.view().root else {
        panic!("a page");
    };
    assert!(page.fill, "the foot stays in view");
    assert_eq!(page.children.len(), 3);
    assert!(matches!(&page.children[1], Element::Scroll(scroll) if scroll.id == "page"));
    let waiting = Harness::<Annotate>::new(start());
    let Element::Page(page) = &waiting.view().root else {
        panic!("a page");
    };
    assert_eq!(page.children.len(), 2, "no foot without an image");
}
