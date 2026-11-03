//! What a drag makes, how marks are drawn, what
//! the JSON and the message say.

use serde_json::json;

use hangar_kit_sdk::ui::{Align, Ink, Mark, Point, Shape as Drawn, default_stroke};

use kit_annotate_tab::answer;
use kit_annotate_tab::model::{self, Color, Counts, Shape, Tool};

const W: f32 = 1000.0;
const H: f32 = 500.0;

fn p(x: f32, y: f32) -> Point {
    Point::new(x, y)
}

#[test]
fn a_point_of_the_canvas_is_the_image_s_pixels_kept_inside_it() {
    // Hangar turns the pointer into the canvas's units; the tab rounds them
    // to a tenth and keeps them inside the image.
    assert_eq!(model::inside(p(500.04, 249.96), W, H), p(500.0, 250.0));
    assert_eq!(model::inside(p(-50.0, 9999.0), W, H), p(0.0, 500.0));
}

#[test]
fn the_pen_s_width_follows_the_image_within_limits() {
    assert_eq!(model::unit(100.0, 100.0), 2.0);
    assert_eq!(model::unit(1000.0, 500.0), 4.0);
    assert_eq!(model::unit(8000.0, 4000.0), 12.0);
    for (w, h) in [
        (100.0, 100.0),
        (1000.0, 500.0),
        (8000.0, 4000.0),
        (1920.0, 1080.0),
    ] {
        assert_eq!(
            model::unit(w, h),
            default_stroke(w, h),
            "the canvas draws with it"
        );
    }
}

#[test]
fn a_drag_makes_the_mark_of_its_tool_and_a_speck_makes_nothing() {
    assert_eq!(
        model::from_drag(
            Tool::Arrow,
            p(10.0, 10.0),
            p(210.0, 110.0),
            &[],
            Color::Red,
            W,
            H
        ),
        Some(Shape::Arrow {
            from: p(10.0, 10.0),
            to: p(210.0, 110.0),
            color: Color::Red
        })
    );
    assert_eq!(
        model::from_drag(
            Tool::Rect,
            p(300.0, 200.0),
            p(100.0, 50.0),
            &[],
            Color::Blue,
            W,
            H
        ),
        Some(Shape::Rect {
            x: 100.0,
            y: 50.0,
            w: 200.0,
            h: 150.0,
            color: Color::Blue
        })
    );
    let stroke = [p(0.0, 0.0), p(3.0, 3.0), p(5.0, 5.0)];
    assert_eq!(
        model::from_drag(
            Tool::Free,
            p(0.0, 0.0),
            p(5.0, 5.0),
            &stroke,
            Color::Green,
            W,
            H
        ),
        Some(Shape::Free {
            points: stroke.to_vec(),
            color: Color::Green
        })
    );
    assert_eq!(
        model::from_drag(
            Tool::Arrow,
            p(10.0, 10.0),
            p(12.0, 11.0),
            &[],
            Color::Red,
            W,
            H
        ),
        None
    );
    assert_eq!(
        model::from_drag(
            Tool::Rect,
            p(10.0, 10.0),
            p(200.0, 11.0),
            &[],
            Color::Red,
            W,
            H
        ),
        None,
        "a flat box is no box"
    );
    assert_eq!(
        model::from_drag(
            Tool::Free,
            p(0.0, 0.0),
            p(0.0, 0.0),
            &[p(0.0, 0.0)],
            Color::Red,
            W,
            H
        ),
        None
    );
    assert_eq!(
        model::from_drag(
            Tool::Note,
            p(0.0, 0.0),
            p(50.0, 50.0),
            &[],
            Color::Red,
            W,
            H
        ),
        None,
        "a note is placed by a click"
    );
}

#[test]
fn the_mark_hangar_sends_is_kept_as_the_tab_s_own() {
    let arrow = Mark::arrow((10.0, 10.0), (210.04, 110.0)).ink(Ink::Red);
    assert_eq!(
        model::from_mark(&arrow, Color::Orange, W, H),
        Some(Shape::Arrow {
            from: p(10.0, 10.0),
            to: p(210.0, 110.0),
            color: Color::Orange
        })
    );
    // A rectangle drawn up and to the left, as Hangar may send it.
    let rect = Mark::rect(300.0, 200.0, -200.0, -150.0);
    assert_eq!(
        model::from_mark(&rect, Color::Blue, W, H),
        Some(Shape::Rect {
            x: 100.0,
            y: 50.0,
            w: 200.0,
            h: 150.0,
            color: Color::Blue
        })
    );
    // A stroke keeps the points that moved enough to show.
    let path = Mark::path([
        p(0.0, 0.0),
        p(0.5, 0.0),
        p(10.0, 0.0),
        p(10.4, 0.0),
        p(20.0, 5.0),
    ]);
    assert_eq!(
        model::from_mark(&path, Color::Green, W, H),
        Some(Shape::Free {
            points: vec![p(0.0, 0.0), p(10.0, 0.0), p(20.0, 5.0)],
            color: Color::Green
        })
    );
    assert_eq!(
        model::from_mark(&Mark::ellipse(0.0, 0.0, 90.0, 90.0), Color::Red, W, H),
        None
    );
    assert_eq!(
        model::from_mark(&Mark::arrow((0.0, 0.0), (3.0, 3.0)), Color::Red, W, H),
        None
    );
}

#[test]
fn a_stroke_keeps_a_point_only_when_the_pointer_moved_enough() {
    let mut points = vec![p(0.0, 0.0)];
    points = model::extend(points, p(0.5, 0.0), W, H);
    assert_eq!(points.len(), 1);
    points = model::extend(points, p(10.0, 0.0), W, H);
    assert_eq!(points.len(), 2);
}

#[test]
fn notes_are_numbered_in_the_order_they_were_placed_and_undo_renumbers() {
    let note = |x, y, text: &str, color| Shape::Note {
        at: p(x, y),
        text: text.into(),
        color,
    };
    let shapes = vec![
        note(1.0, 2.0, "a", Color::Red),
        Shape::Arrow {
            from: p(0.0, 0.0),
            to: p(50.0, 50.0),
            color: Color::Red,
        },
        note(3.0, 4.0, "b", Color::Blue),
    ];
    let numbered: Vec<_> = model::notes_of(&shapes)
        .into_iter()
        .map(|note| (note.n, note.index, note.text))
        .collect();
    assert_eq!(numbered, [(1, 0, "a".to_owned()), (2, 2, "b".to_owned())]);
    let after: Vec<_> = model::notes_of(&shapes[1..])
        .into_iter()
        .map(|note| note.n)
        .collect();
    assert_eq!(after, [1]);
    assert_eq!(
        model::counts(&shapes),
        Counts {
            arrow: 1,
            rect: 0,
            free: 0,
            note: 2
        }
    );
}

#[test]
fn drawing_gives_each_mark_its_colour_and_numbers_the_notes() {
    let shapes = vec![
        Shape::Rect {
            x: 10.0,
            y: 10.0,
            w: 50.0,
            h: 30.0,
            color: Color::Red,
        },
        Shape::Arrow {
            from: p(0.0, 0.0),
            to: p(100.0, 0.0),
            color: Color::Blue,
        },
        Shape::Free {
            points: vec![p(0.0, 0.0), p(5.0, 5.0), p(9.0, 2.0)],
            color: Color::Green,
        },
        Shape::Note {
            at: p(100.0, 100.0),
            text: "too close".into(),
            color: Color::Orange,
        },
        Shape::Note {
            at: p(200.0, 100.0),
            text: String::new(),
            color: Color::Magenta,
        },
    ];
    let marks = model::marks(&shapes, W, H);
    assert_eq!(marks[0], Mark::rect(10.0, 10.0, 50.0, 30.0).ink(Ink::Red));
    assert_eq!(
        marks[1],
        Mark::arrow((0.0, 0.0), (100.0, 0.0)).ink(Ink::Blue)
    );
    assert!(matches!(&marks[2].shape, Drawn::Path { points } if points.len() == 3));
    assert_eq!(marks[2].ink, Ink::Green);
    let texts: Vec<&str> = marks
        .iter()
        .filter_map(|mark| match &mark.shape {
            Drawn::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, ["1", "too close", "2"]);
    // A badge of the note's colour, ringed and numbered in black or white.
    let line = model::unit(W, H);
    assert_eq!(
        marks[3],
        Mark::circle((100.0, 100.0), line * 3.4)
            .ink(Ink::Black)
            .fill(Ink::Orange)
    );
    assert!(matches!(
        &marks[4].shape,
        Drawn::Text {
            bold: true,
            align: Align::Center,
            ..
        }
    ));
    assert_eq!(marks[4].ink, Ink::Orange.contrast());
    assert_eq!(
        marks[5].fill,
        Some(Ink::Orange),
        "the word on a box of its colour"
    );
    assert_eq!(marks.len(), 3 + 3 + 2, "an empty note has no word to write");
}

#[test]
fn a_label_that_would_leave_the_image_goes_to_the_other_side_of_its_badge() {
    let shapes = [Shape::Note {
        at: p(190.0, 50.0),
        text: "long label here".into(),
        color: Color::Red,
    }];
    let marks = model::marks(&shapes, 200.0, 100.0);
    let Drawn::Text {
        at,
        fallback,
        align,
        ..
    } = &marks[2].shape
    else {
        panic!("the label");
    };
    assert!(at.x > 190.0, "to the right of the badge, where it fits");
    assert_eq!(*align, Align::Start);
    assert!(
        fallback.expect("a side to turn to").x < 190.0,
        "to the left of the badge"
    );
}

#[test]
fn black_or_white_ink_whichever_reads() {
    assert_eq!(Ink::White.contrast(), Ink::Black);
    assert_eq!(Ink::Black.contrast(), Ink::White);
    assert_eq!(Color::Blue.ink().contrast(), Ink::White);
    assert_eq!(
        Color::from_id("nope"),
        Color::Red,
        "an unknown colour is the first"
    );
    let hexes: Vec<String> = Color::ALL.iter().map(|color| color.hex()).collect();
    assert_eq!(
        hexes,
        ["#e5484d", "#f08c00", "#2f9e44", "#3b82f6", "#d6409f"],
        "the page's palette"
    );
}

#[test]
fn a_long_word_is_cut_and_kept_on_one_line() {
    assert_eq!(model::clip_label("  too \n close  "), "too close");
    let long = model::clip_label(&"é".repeat(60));
    assert_eq!(long.chars().count(), model::LABEL_MAX_CHARS);
    assert!(long.ends_with('…'));
}

#[test]
fn the_json_describes_each_mark_in_the_written_image_s_pixels() {
    let shapes = [
        Shape::Arrow {
            from: p(10.0, 20.0),
            to: p(110.0, 120.0),
            color: Color::Red,
        },
        Shape::Rect {
            x: 100.0,
            y: 200.0,
            w: 40.0,
            h: 60.0,
            color: Color::Blue,
        },
        Shape::Free {
            points: vec![p(10.0, 10.0), p(30.0, 50.0), p(20.0, 5.0)],
            color: Color::Green,
        },
        Shape::Note {
            at: p(11.0, 13.0),
            text: "x".into(),
            color: Color::Orange,
        },
    ];
    assert_eq!(
        serde_json::to_value(model::describe(&shapes, 0.5)).unwrap(),
        json!([
            { "type": "arrow", "color": "red", "from": [5, 10], "to": [55, 60] },
            { "type": "rectangle", "color": "blue", "x": 50, "y": 100, "width": 20, "height": 30 },
            { "type": "freehand", "color": "green", "x": 5, "y": 3, "width": 10, "height": 22 },
            { "type": "note", "color": "orange", "x": 6, "y": 7 },
        ])
    );
}

fn note(n: usize, x: i64, y: i64, text: &str) -> answer::Note {
    answer::Note {
        n,
        x,
        y,
        text: text.into(),
    }
}

#[test]
fn the_message_names_the_image_the_counts_the_comment_and_each_note() {
    let notes = [note(1, 10, 20, "too close"), note(2, 30, 40, "")];
    let text = answer::message(&answer::Message {
        png_path: ".hangar/annotations/20261031-231512.png",
        json_path: ".hangar/annotations/20261031-231512.json",
        json_saved: true,
        width: 800,
        height: 600,
        source: Some(".hangar/capture.png"),
        comment: "Fix the spacing.",
        notes: &notes,
        counts: Counts {
            arrow: 1,
            rect: 2,
            free: 0,
            note: 2,
        },
        reduced: false,
    });
    assert_eq!(
        text,
        [
            "The person annotated an image for you.",
            "Image: .hangar/annotations/20261031-231512.png (800 × 600 px). Read it to see the marks.",
            "Source: .hangar/capture.png",
            "Marks: 1 arrow, 2 rectangles, 2 notes.",
            "Notes file: .hangar/annotations/20261031-231512.json",
            "Comment: Fix the spacing.",
            "Numbered notes (x, y in the image's pixels):",
            "1. too close (x 10, y 20)",
            "2. (no text) (x 30, y 40)",
        ]
        .join("\n")
    );
}

#[test]
fn the_message_omits_what_is_not_there_says_when_the_image_was_reduced_and_stays_under_the_limit() {
    let base = answer::Message {
        png_path: "a.png",
        json_path: "a.json",
        json_saved: false,
        width: 1,
        height: 1,
        source: None,
        comment: "",
        notes: &[],
        counts: Counts::default(),
        reduced: true,
    };
    let text = answer::message(&base);
    assert!(text.contains("(1 × 1 px, reduced from the original)"));
    assert!(text.contains("Marks: none."));
    for absent in ["Notes file", "Comment", "Source", "Numbered"] {
        assert!(!text.contains(absent), "{absent}");
    }
    let comment = "x".repeat(50_000);
    let long = answer::message(&answer::Message {
        comment: &comment,
        ..base
    });
    assert!(long.chars().count() <= answer::MESSAGE_MAX);
    assert!(long.ends_with('…'));
}

#[test]
fn the_json_file_is_written_as_the_page_writes_it() {
    let record = answer::Record {
        version: 1,
        image: "20261031-231512.png".into(),
        width: 800,
        height: 600,
        source: None,
        created_at: "2026-10-31T22:15:12.000Z".into(),
        comment: String::new(),
        notes: vec![note(1, 120, 240, "too close")],
        marks: model::describe(
            &[Shape::Note {
                at: p(120.0, 240.0),
                text: "too close".into(),
                color: Color::Red,
            }],
            1.0,
        ),
    };
    let text = record.to_text();
    assert!(text.ends_with("}\n"));
    assert!(text.starts_with(
        "{\n  \"version\": 1,\n  \"image\": \"20261031-231512.png\",\n  \"width\": 800,"
    ));
    assert!(text.contains("\"source\": null,\n  \"createdAt\": \"2026-10-31T22:15:12.000Z\","));
    let read: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        read["notes"],
        json!([{ "n": 1, "x": 120, "y": 240, "text": "too close" }])
    );
    assert_eq!(
        read["marks"],
        json!([{ "type": "note", "color": "red", "x": 120, "y": 240 }])
    );
}
