//! The marks on an image: what a drag makes of a tool, how the marks are drawn
//! (the canvas Hangar shows and the PNG it writes alike), and what they add up
//! to. Coordinates are the image's own pixels.

use hangar_kit_sdk::ui::{Align, Ink, Mark, Point, Shape as Drawn, Tool as Pen};
use serde::Serialize;

/// The four tools of the tab, in the order the toolbar shows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Arrow,
    Rect,
    Free,
    Note,
}

impl Tool {
    pub const ALL: [Tool; 4] = [Tool::Arrow, Tool::Rect, Tool::Free, Tool::Note];

    pub fn id(self) -> &'static str {
        match self {
            Tool::Arrow => "arrow",
            Tool::Rect => "rect",
            Tool::Free => "free",
            Tool::Note => "note",
        }
    }

    pub fn from_id(id: &str) -> Option<Tool> {
        Tool::ALL.into_iter().find(|tool| tool.id() == id)
    }

    /// What a drag on the canvas draws with it: a note is a press.
    pub fn pen(self) -> Pen {
        match self {
            Tool::Arrow => Pen::Arrow,
            Tool::Rect => Pen::Rect,
            Tool::Free => Pen::Free,
            Tool::Note => Pen::Point,
        }
    }
}

/// The only colours of the tab: the person's to choose, painted on the image.
/// Hangar's named inks are these very values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Red,
    Orange,
    Green,
    Blue,
    Magenta,
}

impl Color {
    pub const ALL: [Color; 5] = [
        Color::Red,
        Color::Orange,
        Color::Green,
        Color::Blue,
        Color::Magenta,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Color::Red => "red",
            Color::Orange => "orange",
            Color::Green => "green",
            Color::Blue => "blue",
            Color::Magenta => "magenta",
        }
    }

    /// An unknown colour is the first.
    pub fn from_id(id: &str) -> Color {
        Color::ALL
            .into_iter()
            .find(|color| color.id() == id)
            .unwrap_or(Color::Red)
    }

    pub fn ink(self) -> Ink {
        match self {
            Color::Red => Ink::Red,
            Color::Orange => Ink::Orange,
            Color::Green => Ink::Green,
            Color::Blue => Ink::Blue,
            Color::Magenta => Ink::Magenta,
        }
    }

    /// `#e5484d`: the value the page paints.
    pub fn hex(self) -> String {
        let [r, g, b] = self.ink().rgb();
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

/// A mark the person made, in the image's pixels.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Arrow {
        from: Point,
        to: Point,
        color: Color,
    },
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
    },
    Free {
        points: Vec<Point>,
        color: Color,
    },
    /// A numbered note: its number is its place among the notes.
    Note {
        at: Point,
        text: String,
        color: Color,
    },
}

impl Shape {
    pub fn color(&self) -> Color {
        match self {
            Shape::Arrow { color, .. }
            | Shape::Rect { color, .. }
            | Shape::Free { color, .. }
            | Shape::Note { color, .. } => *color,
        }
    }

    pub fn is_note(&self) -> bool {
        matches!(self, Shape::Note { .. })
    }
}

pub const LABEL_MAX_CHARS: usize = 48;

/// The pen's width in image pixels: marks keep their look on a 4K capture and
/// a thumbnail. Hangar's `default_stroke`, the width a canvas draws with.
pub fn unit(width: f32, height: f32) -> f32 {
    (width.max(height) / 250.0).round().clamp(2.0, 12.0)
}

fn round(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

/// A point of the canvas, rounded to a tenth of a pixel and kept inside it.
pub fn inside(point: Point, width: f32, height: f32) -> Point {
    Point::new(
        round(point.x.clamp(0.0, width)),
        round(point.y.clamp(0.0, height)),
    )
}

/// The mark a finished drag makes, or none when it is too small to mean
/// anything.
pub fn from_drag(
    tool: Tool,
    start: Point,
    end: Point,
    points: &[Point],
    color: Color,
    width: f32,
    height: f32,
) -> Option<Shape> {
    let min = unit(width, height) * 2.0;
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    match tool {
        Tool::Arrow => (dx.hypot(dy) >= min * 2.0).then_some(Shape::Arrow {
            from: start,
            to: end,
            color,
        }),
        Tool::Rect => (dx.abs() >= min && dy.abs() >= min).then_some(Shape::Rect {
            x: start.x.min(end.x),
            y: start.y.min(end.y),
            w: dx.abs(),
            h: dy.abs(),
            color,
        }),
        Tool::Free => (points.len() >= 2).then(|| Shape::Free {
            points: points.to_vec(),
            color,
        }),
        Tool::Note => None,
    }
}

/// The mark Hangar sends at the end of a drag, as the tab keeps it: what
/// [`from_drag`] makes of it, in `color`. A shape the tab has no tool for
/// makes nothing.
pub fn from_mark(mark: &Mark, color: Color, width: f32, height: f32) -> Option<Shape> {
    let at = |point: Point| inside(point, width, height);
    match &mark.shape {
        Drawn::Line { from, to, .. } => {
            from_drag(Tool::Arrow, at(*from), at(*to), &[], color, width, height)
        }
        Drawn::Rect { x, y, w, h } => from_drag(
            Tool::Rect,
            at(Point::new(*x, *y)),
            at(Point::new(x + w, y + h)),
            &[],
            color,
            width,
            height,
        ),
        Drawn::Path { points } => {
            let mut kept: Vec<Point> = Vec::new();
            for point in points {
                kept = extend(kept, at(*point), width, height);
            }
            let (first, last) = (*kept.first()?, *kept.last()?);
            from_drag(Tool::Free, first, last, &kept, color, width, height)
        }
        _ => None,
    }
}

/// Adds a point to a stroke only when it moved enough to show.
pub fn extend(mut points: Vec<Point>, point: Point, width: f32, height: f32) -> Vec<Point> {
    if let Some(last) = points.last()
        && (point.x - last.x).hypot(point.y - last.y) < unit(width, height) / 2.0
    {
        return points;
    }
    points.push(point);
    points
}

/// A note among the marks, numbered in the order they were placed.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteRef {
    pub n: usize,
    /// Its place among all the marks.
    pub index: usize,
    pub at: Point,
    pub text: String,
    pub color: Color,
}

pub fn notes_of(shapes: &[Shape]) -> Vec<NoteRef> {
    shapes
        .iter()
        .enumerate()
        .filter_map(|(index, shape)| match shape {
            Shape::Note { at, text, color } => Some((index, *at, text.clone(), *color)),
            _ => None,
        })
        .enumerate()
        .map(|(i, (index, at, text, color))| NoteRef {
            n: i + 1,
            index,
            at,
            text,
            color,
        })
        .collect()
}

/// How many of each, for one sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub arrow: usize,
    pub rect: usize,
    pub free: usize,
    pub note: usize,
}

pub fn counts(shapes: &[Shape]) -> Counts {
    let mut total = Counts::default();
    for shape in shapes {
        match shape {
            Shape::Arrow { .. } => total.arrow += 1,
            Shape::Rect { .. } => total.rect += 1,
            Shape::Free { .. } => total.free += 1,
            Shape::Note { .. } => total.note += 1,
        }
    }
    total
}

/// A note's word as it is written beside its badge: on one line, cut at
/// [`LABEL_MAX_CHARS`] with an ellipsis.
pub fn clip_label(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = flat.chars().collect();
    if chars.len() > LABEL_MAX_CHARS {
        let mut cut: String = chars[..LABEL_MAX_CHARS - 1].iter().collect();
        cut.push('…');
        cut
    } else {
        flat
    }
}

/// The marks as the canvas draws them, on screen and in the PNG alike: an
/// arrow with its head, a rectangle, a stroke, and for a note a badge of its
/// colour with its number, and its word on a box beside it, turned to the
/// badge's other side at the image's right edge.
pub fn marks(shapes: &[Shape], width: f32, height: f32) -> Vec<Mark> {
    let line = unit(width, height);
    let mut n = 0;
    let mut drawn = Vec::new();
    for shape in shapes {
        let ink = shape.color().ink();
        match shape {
            Shape::Arrow { from, to, .. } => drawn.push(Mark::arrow(*from, *to).ink(ink)),
            Shape::Rect { x, y, w, h, .. } => drawn.push(Mark::rect(*x, *y, *w, *h).ink(ink)),
            Shape::Free { points, .. } => drawn.push(Mark::path(points.iter().copied()).ink(ink)),
            Shape::Note { at, text, .. } => {
                n += 1;
                let radius = line * 3.4;
                drawn.push(Mark::circle(*at, radius).ink(ink.contrast()).fill(ink));
                drawn.push(
                    Mark::text(*at, n.to_string(), round(radius * 1.25))
                        .bold()
                        .align(Align::Center)
                        .ink(ink.contrast()),
                );
                let label = clip_label(text);
                if !label.is_empty() {
                    let size = radius * 1.05;
                    let pad = size * 0.35;
                    drawn.push(
                        Mark::text((at.x + radius + pad, at.y), label, round(size))
                            .fill(ink)
                            .ink(ink.contrast())
                            .fallback((at.x - radius - pad, at.y)),
                    );
                }
            }
        }
    }
    drawn
}

/// A mark as the JSON file gives it, in the written image's pixels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Described {
    Arrow {
        color: Color,
        from: [i64; 2],
        to: [i64; 2],
    },
    Rectangle {
        color: Color,
        x: i64,
        y: i64,
        width: i64,
        height: i64,
    },
    Freehand {
        color: Color,
        x: i64,
        y: i64,
        width: i64,
        height: i64,
    },
    Note {
        color: Color,
        x: i64,
        y: i64,
    },
}

/// `value` in the written image's pixels, rounded as `Math.round` does.
pub fn px(value: f32, scale: f32) -> i64 {
    (value as f64 * scale as f64 + 0.5).floor() as i64
}

/// The marks as the JSON file gives them, scaled and rounded to the written
/// image.
pub fn describe(shapes: &[Shape], scale: f32) -> Vec<Described> {
    let px = |value: f32| px(value, scale);
    shapes
        .iter()
        .map(|shape| match shape {
            Shape::Arrow { from, to, color } => Described::Arrow {
                color: *color,
                from: [px(from.x), px(from.y)],
                to: [px(to.x), px(to.y)],
            },
            Shape::Rect { x, y, w, h, color } => Described::Rectangle {
                color: *color,
                x: px(*x),
                y: px(*y),
                width: px(*w),
                height: px(*h),
            },
            Shape::Free { points, color } => {
                let low_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let low_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let high_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let high_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                let (x, y) = (px(low_x), px(low_y));
                Described::Freehand {
                    color: *color,
                    x,
                    y,
                    width: px(high_x) - x,
                    height: px(high_y) - y,
                }
            }
            Shape::Note { at, color, .. } => Described::Note {
                color: *color,
                x: px(at.x),
                y: px(at.y),
            },
        })
        .collect()
}
