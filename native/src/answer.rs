//! What leaves the tab for the agent: the message `panes.send` carries, and
//! the small JSON file beside the image. In English whatever the tab's
//! language, because an agent reads it; the person's own words (notes,
//! comment) go through as they wrote them. `tabs/annotate/answer.js`, ported.

use serde::Serialize;

use crate::model::{Counts, Described};

/// `panes.send` takes 20 000 characters at most.
pub const MESSAGE_MAX: usize = 20_000;

fn plural(n: usize, one: &str, other: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { other })
}

pub fn marks_sentence(counts: &Counts) -> String {
    let mut parts = Vec::new();
    if counts.arrow > 0 {
        parts.push(plural(counts.arrow, "arrow", "arrows"));
    }
    if counts.rect > 0 {
        parts.push(plural(counts.rect, "rectangle", "rectangles"));
    }
    if counts.free > 0 {
        parts.push(plural(counts.free, "freehand stroke", "freehand strokes"));
    }
    if counts.note > 0 {
        parts.push(plural(counts.note, "note", "notes"));
    }
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(", ")
    }
}

/// A numbered note, in the written image's pixels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Note {
    pub n: usize,
    pub x: i64,
    pub y: i64,
    pub text: String,
}

/// The JSON file: the image's name and size, the comment, the numbered notes
/// and a line per mark, all in the written image's pixels.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub version: u32,
    pub image: String,
    pub width: u32,
    pub height: u32,
    pub source: Option<String>,
    pub created_at: String,
    pub comment: String,
    pub notes: Vec<Note>,
    pub marks: Vec<Described>,
}

impl Record {
    /// As the page writes it: two spaces of indent, a line break at the end.
    pub fn to_text(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).unwrap_or_default();
        text.push('\n');
        text
    }
}

/// What the message says.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    pub png_path: &'a str,
    pub json_path: &'a str,
    pub json_saved: bool,
    pub width: u32,
    pub height: u32,
    pub source: Option<&'a str>,
    pub comment: &'a str,
    pub notes: &'a [Note],
    pub counts: Counts,
    pub reduced: bool,
}

pub fn message(data: &Message) -> String {
    let mut lines = vec![
        "The person annotated an image for you.".to_owned(),
        format!(
            "Image: {} ({} × {} px{}). Read it to see the marks.",
            data.png_path,
            data.width,
            data.height,
            if data.reduced {
                ", reduced from the original"
            } else {
                ""
            }
        ),
    ];
    if let Some(source) = data.source.filter(|source| !source.is_empty()) {
        lines.push(format!("Source: {source}"));
    }
    lines.push(format!("Marks: {}.", marks_sentence(&data.counts)));
    if data.json_saved {
        lines.push(format!("Notes file: {}", data.json_path));
    }
    if !data.comment.is_empty() {
        lines.push(format!("Comment: {}", data.comment));
    }
    if !data.notes.is_empty() {
        lines.push("Numbered notes (x, y in the image's pixels):".to_owned());
        for note in data.notes {
            let text = if note.text.is_empty() {
                "(no text)"
            } else {
                note.text.as_str()
            };
            lines.push(format!("{}. {text} (x {}, y {})", note.n, note.x, note.y));
        }
    }
    let text = lines.join("\n");
    if text.chars().count() > MESSAGE_MAX {
        let mut cut: String = text.chars().take(MESSAGE_MAX - 1).collect();
        cut.push('…');
        cut
    } else {
        text
    }
}
