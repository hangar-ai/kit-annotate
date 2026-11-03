//! Annotate, native: a capture drawn on with an arrow, a rectangle, a
//! freehand stroke and numbered notes, written as a PNG and a JSON beside it,
//! then told to the agent that asked. The page in `tabs/annotate/` is the
//! same tab for Hangar 1.x; this is `controller.js` and `view.js` as a
//! [`Kit`], with `logic.js`, `model.js` and `answer.js` in their own modules.
//!
//! What differs from the page, because a native tab works so:
//!
//! - The capture is never read into the module: the canvas shows it where it
//!   is (`ImageSource::File`), its size comes from `image.size`, and Hangar
//!   draws the PNG itself (`canvas.export`), scaled down as the page did.
//! - A picture from outside the workspace comes from the file chooser
//!   (`files.pick`, under `files.read`) or the clipboard (`ui.pasteImage`,
//!   Ctrl+V or a button), held by the tab. A file dropped on the tab is not
//!   heard: the window hears a drop, not where it lands.
//! - Hangar draws a drag while it moves and sends the finished mark: the tab
//!   keeps it or drops it as `model.fromDrag` did.
//! - Ctrl+Z reaches the tab while a field has the focus (iced's fields have no
//!   undo): the tab ignores it after typing, until a press elsewhere.

pub mod answer;
pub mod logic;
pub mod model;

use hangar_kit_sdk::api::{
    CanvasExport, Done, Exported, HeldImage, ImageInfo, Picked, Queued, TabParamsAnswer, Theme,
};
use hangar_kit_sdk::prelude::*;

use model::{Color, Shape, Tool};

/// What the tab shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// No image: the tab waits for one.
    Empty,
    /// The image the agent named is being looked at.
    Loading,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    Denied,
    Info,
    Error,
}

/// What is wrong with opening, or what opening is doing.
#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub kind: AlertKind,
    pub title: Option<String>,
    pub text: String,
    /// A path to read again.
    pub retry: Option<String>,
}

impl Alert {
    fn error(text: String) -> Self {
        Alert {
            kind: AlertKind::Error,
            title: None,
            text,
            retry: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Success,
    Warning,
    Error,
}

/// What the last send came to, said in the foot.
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    pub kind: StatusKind,
    pub text: String,
    pub png: Option<String>,
}

/// The image open, where it is, and its size in pixels: the canvas's units.
#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    pub source: ImageSource,
    /// Its file's name; empty for a paste.
    pub name: String,
    pub width: u32,
    pub height: u32,
}

/// Where an image being opened comes from.
#[derive(Debug, Clone, PartialEq)]
pub struct Origin {
    source: ImageSource,
    name: String,
    /// Its path in the workspace, when it has one.
    path: Option<String>,
    /// Named by the agent (the `path` param): its failures are said by code.
    by_path: bool,
}

/// A send under way, frozen when Send was pressed: what is written and told
/// does not change if the person keeps drawing.
struct Delivery {
    export: CanvasExport,
    names: Vec<String>,
    tried: usize,
    at_ms: i64,
    shapes: Vec<Shape>,
    source: Option<String>,
    comment: String,
    pane: String,
    png: String,
    json: String,
    exported: Option<Exported>,
    json_problem: Option<String>,
}

pub struct Annotate {
    t: Strings,
    pub phase: Phase,
    pub notice: Option<Alert>,
    pub image: Option<Picture>,
    /// The workspace path of the image open, when it has one: what a new
    /// `tab.params` is told apart by, and what the JSON names as its source.
    pub source: Option<String>,
    pub shapes: Vec<Shape>,
    pub tool: Tool,
    pub color: Color,
    /// None: the whole image fits the panel.
    pub zoom: Option<f32>,
    /// The scale Hangar fits the canvas at, while there is no zoom.
    pub fit: f32,
    pub comment: String,
    /// As typed: trimmed where it is used, so that a space can be typed.
    pub target: String,
    pub caller: Option<String>,
    pub busy: bool,
    pub status: Option<Status>,
    /// A field was typed in last, and so has the focus as far as the tab can
    /// tell: Ctrl+Z is then the field's, not an undo of a mark.
    pub typing: bool,
    opening: u32,
    delivery: Option<Delivery>,
}

#[derive(Clone)]
pub enum Message {
    Sized {
        mine: u32,
        origin: Origin,
        result: Result<ImageInfo, KitError>,
    },
    Choose,
    Picked(Result<Option<Picked>, KitError>),
    Paste,
    Pasted(u32, Result<HeldImage, KitError>),
    Retry(String),
    Another,
    Replace(bool),
    Params(TabParamsAnswer),
    Theme(Theme),
    Tool(Tool),
    Color(Color),
    ZoomIn,
    ZoomOut,
    Fit,
    Fitted(f32),
    Drew(Mark),
    Pointed(Point),
    /// The Undo button.
    Undo,
    /// Ctrl+Z.
    UndoKey,
    NoteText(usize, String),
    Comment(String),
    Target(String),
    Send,
    Exported(Result<Exported, KitError>),
    JsonWritten(Result<Done, KitError>),
    Told(Result<Queued, KitError>),
}

fn reason(error: &KitError) -> String {
    if error.message.trim().is_empty() {
        "unknown error".to_owned()
    } else {
        error.message.clone()
    }
}

impl Annotate {
    /// The image, when it is shown and can be drawn on.
    pub fn ready(&self) -> Option<&Picture> {
        self.image.as_ref().filter(|_| self.phase == Phase::Ready)
    }

    /// The zoom the canvas is drawn at: the one chosen, or the fitted one.
    pub fn effective_zoom(&self) -> f32 {
        self.zoom.unwrap_or(self.fit)
    }

    fn after_failure(&mut self) {
        self.phase = if self.image.is_some() {
            Phase::Ready
        } else {
            Phase::Empty
        };
    }

    /// An image of the workspace, by its path.
    fn load_path(&mut self, path: String, cx: &mut Context<Message>) {
        self.opening += 1;
        let mine = self.opening;
        self.phase = Phase::Loading;
        self.notice = Some(Alert {
            kind: AlertKind::Info,
            title: None,
            text: self.t.tv("loading", &[("path", &path)]),
            retry: None,
        });
        let origin = Origin {
            source: ImageSource::File(path.clone()),
            name: logic::base_name(&path),
            path: Some(path),
            by_path: true,
        };
        cx.call(
            api::ImageSize {
                source: origin.source.clone(),
            },
            move |result| Message::Sized {
                mine,
                origin,
                result,
            },
        );
    }

    fn show(&mut self, origin: Origin, width: u32, height: u32) {
        self.image = Some(Picture {
            source: origin.source,
            name: origin.name,
            width,
            height,
        });
        self.source = origin.path;
        self.shapes.clear();
        self.zoom = None;
        self.status = None;
        self.notice = None;
        self.typing = false;
        self.phase = Phase::Ready;
    }

    /// Back to nothing open: what "Another image" confirms.
    fn close(&mut self) {
        self.opening += 1;
        self.image = None;
        self.source = None;
        self.shapes.clear();
        self.zoom = None;
        self.status = None;
        self.notice = None;
        self.typing = false;
        self.phase = Phase::Empty;
    }

    /// A pasted or chosen image does not replace marks: they would be lost.
    fn keeps_marks(&mut self) -> bool {
        if self.shapes.is_empty() {
            return false;
        }
        self.status = Some(Status {
            kind: StatusKind::Warning,
            text: self.t.t("keep_marks"),
            png: None,
        });
        true
    }

    fn sized(&mut self, origin: Origin, result: Result<ImageInfo, KitError>) {
        match result {
            Ok(info) if info.width > 0 && info.height > 0 => {
                self.show(origin, info.width, info.height)
            }
            Err(error) if origin.by_path => {
                let path = origin.path.unwrap_or_default();
                let t = &self.t;
                self.notice = Some(if error.is(KitError::DENIED) {
                    Alert {
                        kind: AlertKind::Denied,
                        title: Some(t.t("denied_title")),
                        text: t.t("denied_text"),
                        retry: None,
                    }
                } else if error.is(KitError::NOT_FOUND) {
                    Alert::error(t.tv("not_found", &[("path", &path)]))
                } else if error.is(KitError::UNSUPPORTED) {
                    Alert::error(t.tv("unsupported", &[("path", &path)]))
                } else {
                    Alert {
                        retry: Some(path.clone()),
                        ..Alert::error(t.tv(
                            "read_failed",
                            &[("path", &path), ("reason", &reason(&error))],
                        ))
                    }
                });
                self.after_failure();
            }
            Ok(_) | Err(_) => {
                self.notice = Some(Alert::error(self.t.t("bad_image")));
                self.after_failure();
            }
        }
    }

    fn picked(&mut self, picked: Picked, cx: &mut Context<Message>) {
        if self.keeps_marks() {
            return;
        }
        if picked.size > logic::INPUT_MAX_BYTES {
            self.notice = Some(Alert::error(self.t.t("too_large")));
            return;
        }
        let source = match (&picked.path, &picked.held) {
            (Some(path), _) => ImageSource::File(path.clone()),
            (None, Some(held)) => ImageSource::Held(held.clone()),
            (None, None) => {
                self.notice = Some(Alert::error(self.t.t("bad_image")));
                return;
            }
        };
        self.opening += 1;
        let mine = self.opening;
        let origin = Origin {
            source: source.clone(),
            name: picked.name,
            path: picked.path,
            by_path: false,
        };
        cx.call(api::ImageSize { source }, move |result| Message::Sized {
            mine,
            origin,
            result,
        });
    }

    /// The tab was asked for again (`tab.params`): the latest to ask is the
    /// one to answer. The same image keeps its marks, and so does any image
    /// that has some.
    fn receive(&mut self, answer: TabParamsAnswer, cx: &mut Context<Message>) {
        if let Some(pane) = answer.caller.map(|caller| caller.pane)
            && !pane.trim().is_empty()
        {
            self.caller = Some(pane.clone());
            self.target = pane;
        }
        if let Some(path) = logic::clean_path(answer.params.get("path").map(String::as_str))
            && self.source.as_deref() != Some(path.as_str())
            && self.shapes.is_empty()
        {
            self.load_path(path, cx);
        }
    }

    fn zoom_to(&mut self, zoom: Option<f32>) {
        self.zoom = zoom.map(logic::clamp_zoom);
    }

    fn undo(&mut self) {
        self.shapes.pop();
    }

    /// A click with the note tool: a numbered mark whose word is written in
    /// the list, its field given the keyboard.
    fn add_note(&mut self, at: Point, cx: &mut Context<Message>) {
        let Some(picture) = self.ready() else { return };
        let at = model::inside(at, picture.width as f32, picture.height as f32);
        self.shapes.push(Shape::Note {
            at,
            text: String::new(),
            color: self.color,
        });
        self.status = None;
        let n = model::notes_of(&self.shapes).len();
        cx.focus(format!("note-{n}"));
        self.typing = true;
    }

    fn set_note_text(&mut self, n: usize, text: String) {
        let Some(note) = model::notes_of(&self.shapes)
            .into_iter()
            .find(|note| note.n == n)
        else {
            return;
        };
        if let Some(Shape::Note { text: word, .. }) = self.shapes.get_mut(note.index) {
            *word = logic::clip(&text, logic::NOTE_MAX, false);
        }
    }

    // ------------------------------------------------------------ Sending.

    /// Writes the PNG then the notes, under a name no file has, and tells the
    /// agent.
    fn send(&mut self, cx: &mut Context<Message>) {
        if self.busy {
            return;
        }
        let Some(picture) = self.ready().cloned() else {
            return;
        };
        self.busy = true;
        self.status = None;
        self.typing = false;
        let at_ms = cx.now();
        let base = logic::stamp(&format::local_time(at_ms, cx.utc_offset()));
        let (width, height) = (picture.width as f32, picture.height as f32);
        self.delivery = Some(Delivery {
            export: CanvasExport {
                path: String::new(),
                width,
                height,
                background: Some(picture.source.clone()),
                marks: model::marks(&self.shapes, width, height),
                stroke: None,
            },
            names: logic::names(&base, 5),
            tried: 0,
            at_ms,
            shapes: self.shapes.clone(),
            source: self.source.clone(),
            comment: self.comment.trim().to_owned(),
            pane: self.target.trim().to_owned(),
            png: String::new(),
            json: String::new(),
            exported: None,
            json_problem: None,
        });
        self.export_next(cx);
    }

    fn export_next(&mut self, cx: &mut Context<Message>) {
        let Some(delivery) = &mut self.delivery else {
            return;
        };
        let name = &delivery.names[delivery.tried];
        delivery.png = format!("{}/{name}.png", logic::FOLDER);
        delivery.json = format!("{}/{name}.json", logic::FOLDER);
        delivery.export.path = delivery.png.clone();
        cx.call(delivery.export.clone(), Message::Exported);
    }

    fn finish(&mut self, kind: StatusKind, text: String, png: Option<String>) {
        self.status = Some(Status { kind, text, png });
        self.busy = false;
        self.delivery = None;
    }

    fn exported(&mut self, result: Result<Exported, KitError>, cx: &mut Context<Message>) {
        let Some(delivery) = &mut self.delivery else {
            return;
        };
        let exported = match result {
            Ok(exported) => exported,
            Err(error) if error.is(KitError::CONFLICT) => {
                delivery.tried += 1;
                if delivery.tried < delivery.names.len() {
                    return self.export_next(cx);
                }
                let text = self.t.tv("write_failed", &[("reason", "name taken")]);
                return self.finish(StatusKind::Error, text, None);
            }
            Err(error) => {
                let t = &self.t;
                let text = if error.is(KitError::DENIED) {
                    t.t("write_denied")
                } else if error.is(KitError::UNKNOWN) {
                    t.t("write_unknown")
                } else if error.is(KitError::FAILED) && error.message.contains("even at a quarter")
                {
                    t.t("too_heavy")
                } else {
                    t.tv("write_failed", &[("reason", &reason(&error))])
                };
                return self.finish(StatusKind::Error, text, None);
            }
        };
        let scale = exported.scale;
        let notes = record_notes(&delivery.shapes, scale);
        let used = logic::base_name(&delivery.png);
        let record = answer::Record {
            version: 1,
            image: used,
            width: exported.width,
            height: exported.height,
            source: delivery.source.clone(),
            created_at: logic::iso(delivery.at_ms),
            comment: delivery.comment.clone(),
            notes,
            marks: model::describe(&delivery.shapes, scale),
        };
        delivery.exported = Some(exported);
        cx.call(
            api::FilesWrite {
                path: delivery.json.clone(),
                text: record.to_text(),
                expected: None,
            },
            Message::JsonWritten,
        );
    }

    fn json_written(&mut self, result: Result<Done, KitError>, cx: &mut Context<Message>) {
        let Some(delivery) = &mut self.delivery else {
            return;
        };
        delivery.json_problem = result.err().map(|error| reason(&error));
        let png = delivery.png.clone();
        if delivery.pane.is_empty() {
            return match delivery.json_problem.clone() {
                None => {
                    let text = self.t.tv("saved", &[("png", &png)]);
                    self.finish(StatusKind::Success, text, Some(png))
                }
                Some(problem) => {
                    let text = self.t.tv("json_failed", &[("reason", &problem)]);
                    self.finish(StatusKind::Warning, text, Some(png))
                }
            };
        }
        let Some(exported) = &delivery.exported else {
            return;
        };
        let notes = record_notes(&delivery.shapes, exported.scale);
        let text = answer::message(&answer::Message {
            png_path: &delivery.png,
            json_path: &delivery.json,
            json_saved: delivery.json_problem.is_none(),
            width: exported.width,
            height: exported.height,
            source: delivery.source.as_deref(),
            comment: &delivery.comment,
            notes: &notes,
            counts: model::counts(&delivery.shapes),
            reduced: exported.scale < 1.0,
        });
        cx.call(
            api::PanesSend {
                pane: delivery.pane.clone(),
                text,
            },
            Message::Told,
        );
    }

    fn told(&mut self, result: Result<Queued, KitError>) {
        let Some(delivery) = &self.delivery else {
            return;
        };
        let (png, pane) = (delivery.png.clone(), delivery.pane.clone());
        let problem = delivery.json_problem.clone();
        let t = &self.t;
        let (kind, text) = match result {
            Err(error) if error.is(KitError::DENIED) => {
                (StatusKind::Warning, t.tv("send_denied", &[("png", &png)]))
            }
            Err(error) if error.is(KitError::NOT_FOUND) => (
                StatusKind::Warning,
                t.tv("send_not_found", &[("png", &png), ("pane", &pane)]),
            ),
            Err(error) if error.is(KitError::UNAVAILABLE) => (
                StatusKind::Warning,
                t.tv("send_unavailable", &[("png", &png), ("pane", &pane)]),
            ),
            Err(error) => (
                StatusKind::Warning,
                t.tv(
                    "send_failed",
                    &[("png", &png), ("pane", &pane), ("reason", &reason(&error))],
                ),
            ),
            Ok(_) => match problem {
                Some(problem) => (
                    StatusKind::Warning,
                    t.tv("json_failed", &[("reason", &problem)]),
                ),
                None => (
                    StatusKind::Success,
                    t.tv("sent", &[("pane", &pane), ("png", &png)]),
                ),
            },
        };
        self.finish(kind, text, Some(png));
    }

    // ------------------------------------------------------------ The page.

    fn notice_box(&self, notice: &Alert) -> Element<Message> {
        let t = &self.t;
        match notice.kind {
            AlertKind::Denied => denied(notice.title.clone().unwrap_or_default())
                .text(notice.text.clone())
                .action(button(t.t("paste")).primary().on_press(Message::Paste))
                .into(),
            AlertKind::Info => loading(notice.text.clone()).into(),
            AlertKind::Error => {
                let mut banner = banner(Notice::Error, notice.text.clone());
                if let Some(path) = &notice.retry {
                    banner = banner.action(
                        button(t.t("retry"))
                            .small()
                            .on_press(Message::Retry(path.clone())),
                    );
                }
                banner.into()
            }
        }
    }

    fn header(&self) -> Element<Message> {
        let t = &self.t;
        let mut head = header(t.t("tab"));
        if let Some(picture) = self.ready() {
            let size = t.tv(
                "image_size",
                &[
                    ("w", &picture.width.to_string()),
                    ("h", &picture.height.to_string()),
                ],
            );
            let line = if picture.name.is_empty() {
                size.clone()
            } else {
                format!("{} · {size}", picture.name)
            };
            let whole = if picture.name.is_empty() {
                size
            } else {
                picture.name.clone()
            };
            head = head
                .state(tooltip(text(line).small().dim().truncate(), whole))
                .action(button(t.t("another")).small().on_press(Message::Another));
        }
        // A container is as wide as it is told: the header fills it.
        container(head)
            .padding([0.0, 12.0])
            .width(Length::Fill)
            .into()
    }

    fn toolbar(&self) -> Vec<Element<Message>> {
        let t = &self.t;
        let mut tools: Vec<Element<Message>> = Tool::ALL
            .into_iter()
            .map(|tool| {
                filter(t.t(&format!("tool_{}", tool.id())), self.tool == tool)
                    .on_press(Message::Tool(tool))
                    .into()
            })
            .collect();
        tools.push(space().width(Length::Px(4.0)).into());
        tools.extend(Color::ALL.into_iter().map(|color| {
            swatch(color.ink(), t.t(&format!("color_{}", color.id())))
                .selected(self.color == color)
                .on_press(Message::Color(color))
                .into()
        }));
        let percent = (self.effective_zoom() * 100.0).round().to_string();
        let second = row![
            button(t.t("undo"))
                .small()
                .icon(Icon::Undo)
                .on_press_maybe((!self.shapes.is_empty()).then_some(Message::Undo)),
            space().width(Length::Px(8.0)),
            icon_button(Icon::Minus, t.t("zoom_out"))
                .small()
                .on_press(Message::ZoomOut),
            container(
                text(t.tv("zoom_value", &[("n", &percent)]))
                    .small()
                    .dim()
                    .align(Align::Center)
            )
            .width(Length::Px(44.0)),
            icon_button(Icon::Plus, t.t("zoom_in"))
                .small()
                .on_press(Message::ZoomIn),
            button(t.t("zoom_fit"))
                .small()
                .ghost()
                .on_press(Message::Fit),
        ]
        .spacing(6.0)
        .align(Align::Center)
        .wrap();
        vec![
            row(tools).spacing(6.0).align(Align::Center).wrap().into(),
            second.into(),
            text(t.t(&format!("tip_{}", self.tool.id())))
                .small()
                .dim()
                .into(),
        ]
    }

    fn stage(&self, picture: &Picture) -> Element<Message> {
        let (width, height) = (picture.width as f32, picture.height as f32);
        let board = canvas("shot", width, height)
            .background(picture.source.clone())
            .marks(model::marks(&self.shapes, width, height))
            .tool(Some(self.tool.pen()))
            .ink(self.color.ink())
            .description(self.t.t("canvas"))
            .on_draw(Message::Drew)
            .on_point(Message::Pointed);
        let inner: Element<Message> = match self.zoom {
            None => board.on_fit(Message::Fitted).into(),
            Some(zoom) => scroll(board.zoom(Some(zoom)))
                .axis(Axis::Both)
                .height(Length::Px((height * zoom).min(480.0) + 12.0))
                .into(),
        };
        container(inner)
            .surface(Surface::Well)
            .border()
            .radius(6.0)
            .width(Length::Fill)
            .align_x(Align::Center)
            .into()
    }

    fn notes(&self) -> Element<Message> {
        let t = &self.t;
        let notes = model::notes_of(&self.shapes);
        let children: Vec<Element<Message>> = if notes.is_empty() {
            vec![text(t.t("no_notes")).small().dim().into()]
        } else {
            notes
                .iter()
                .map(|note| {
                    let n = note.n;
                    row![
                        container(text(n.to_string()).numeric()).width(Length::Px(20.0)),
                        text_input(t.t("note_placeholder"), note.text.clone())
                            .id(format!("note-{n}"))
                            .on_input(move |text| Message::NoteText(n, text)),
                    ]
                    .spacing(6.0)
                    .align(Align::Center)
                    .into()
                })
                .collect()
        };
        section(t.t("notes_title"), children)
            .count(notes.len().to_string())
            .into()
    }

    fn fields(&self) -> Element<Message> {
        let t = &self.t;
        let target = self.target.trim();
        let hint = match &self.caller {
            Some(caller) if target == caller => t.tv("target_caller", &[("pane", caller)]),
            _ if !target.is_empty() => String::new(),
            _ => t.t("target_none"),
        };
        let mut pane = text_input(t.t("target_placeholder"), self.target.clone())
            .id("target")
            .label(t.t("target_label"))
            .on_input(Message::Target);
        if !hint.is_empty() {
            pane = pane.hint(hint);
        }
        col![
            text_area("comment", self.comment.clone())
                .label(t.t("comment_label"))
                .placeholder(t.t("comment_placeholder"))
                .height(64.0)
                .on_input(Message::Comment),
            pane,
        ]
        .spacing(12.0)
        .into()
    }

    fn foot(&self) -> Element<Message> {
        let t = &self.t;
        let label = if self.busy {
            t.t("sending")
        } else if self.target.trim().is_empty() {
            t.t("save")
        } else {
            t.t("send")
        };
        let mut said = text(
            self.status
                .as_ref()
                .map(|status| status.text.clone())
                .unwrap_or_default(),
        )
        .small();
        if self
            .status
            .as_ref()
            .is_some_and(|status| status.kind == StatusKind::Error)
        {
            said = said.tone(Tone::Danger);
        }
        col![
            divider(),
            container(
                row![
                    container(said).width(Length::Fill),
                    button(label)
                        .primary()
                        .small()
                        .on_press_maybe((!self.busy).then_some(Message::Send)),
                ]
                .spacing(8.0)
                .align(Align::Center)
            )
            .padding([6.0, 12.0])
            .width(Length::Fill),
        ]
        .into()
    }
}

/// The notes as the JSON and the message give them, in the written image's
/// pixels.
fn record_notes(shapes: &[Shape], scale: f32) -> Vec<answer::Note> {
    model::notes_of(shapes)
        .into_iter()
        .map(|note| answer::Note {
            n: note.n,
            x: model::px(note.at.x, scale),
            y: model::px(note.at.y, scale),
            text: note.text,
        })
        .collect()
}

impl Kit for Annotate {
    type Message = Message;

    fn init(start: &Start, cx: &mut Context<Message>) -> Self {
        let mut tab = Annotate {
            t: logic::strings(&start.theme.locale),
            phase: Phase::Empty,
            notice: None,
            image: None,
            source: None,
            shapes: Vec::new(),
            tool: Tool::Arrow,
            color: Color::Red,
            zoom: None,
            fit: 1.0,
            comment: String::new(),
            target: String::new(),
            caller: None,
            busy: false,
            status: None,
            typing: false,
            opening: 0,
            delivery: None,
        };
        if let Some(caller) = &start.tab.caller
            && !caller.pane.trim().is_empty()
        {
            tab.caller = Some(caller.pane.clone());
            tab.target = caller.pane.clone();
        }
        if let Some(path) = logic::clean_path(start.tab.params.get("path").map(String::as_str)) {
            tab.load_path(path, cx);
        }
        tab
    }

    fn update(&mut self, message: Message, cx: &mut Context<Message>) {
        match message {
            Message::Sized {
                mine,
                origin,
                result,
            } => {
                if mine == self.opening {
                    self.sized(origin, result);
                }
            }
            Message::Choose => cx.call(
                api::FilesPick {
                    title: Some(self.t.t("choose")),
                    extensions: logic::PICKABLE.map(String::from).to_vec(),
                },
                Message::Picked,
            ),
            Message::Picked(Ok(Some(picked))) => self.picked(picked, cx),
            Message::Picked(Ok(None)) => {}
            Message::Picked(Err(error)) if error.is(KitError::DENIED) => {
                self.notice = Some(Alert {
                    kind: AlertKind::Denied,
                    title: Some(self.t.t("denied_title")),
                    text: self.t.t("denied_text"),
                    retry: None,
                });
            }
            // Another chooser is open already.
            Message::Picked(Err(error)) if error.is(KitError::BUSY) => {}
            Message::Picked(Err(error)) => {
                self.notice = Some(Alert::error(
                    self.t.tv("pick_failed", &[("reason", &reason(&error))]),
                ));
            }
            Message::Paste => {
                if self.keeps_marks() {
                    return;
                }
                self.opening += 1;
                let mine = self.opening;
                cx.call(api::PasteImage {}, move |result| {
                    Message::Pasted(mine, result)
                });
            }
            Message::Pasted(mine, _) if mine != self.opening => {}
            Message::Pasted(_, Ok(held)) => {
                let origin = Origin {
                    source: ImageSource::Held(held.held),
                    name: String::new(),
                    path: None,
                    by_path: false,
                };
                self.sized(
                    origin,
                    Ok(ImageInfo {
                        width: held.width,
                        height: held.height,
                    }),
                );
            }
            Message::Pasted(_, Err(error)) => {
                self.notice = Some(Alert::error(if error.is(KitError::NOT_FOUND) {
                    self.t.t("paste_none")
                } else {
                    self.t.tv("paste_failed", &[("reason", &reason(&error))])
                }));
                // A paste while the agent's image was looked for replaced
                // that look: the tab is not left loading.
                self.after_failure();
            }
            Message::Retry(path) => self.load_path(path, cx),
            Message::Another => {
                self.typing = false;
                if self.shapes.is_empty() {
                    self.close();
                } else {
                    let t = &self.t;
                    cx.confirm(
                        api::Confirm {
                            message: t.t("another_ask"),
                            confirm: Some(t.t("another_yes")),
                            cancel: Some(t.t("another_no")),
                            danger: true,
                        },
                        Message::Replace,
                    );
                }
            }
            Message::Replace(true) => self.close(),
            Message::Replace(false) => {}
            Message::Params(answer) => self.receive(answer, cx),
            Message::Theme(theme) => self.t.set_locale(logic::pick_lang(&theme.locale)),
            Message::Tool(tool) => {
                self.tool = tool;
                self.typing = false;
            }
            Message::Color(color) => {
                self.color = color;
                self.typing = false;
            }
            Message::ZoomIn => {
                self.typing = false;
                self.zoom_to(Some(logic::step_zoom(self.effective_zoom(), 1)));
            }
            Message::ZoomOut => {
                self.typing = false;
                self.zoom_to(Some(logic::step_zoom(self.effective_zoom(), -1)));
            }
            Message::Fit => {
                self.typing = false;
                self.zoom_to(None);
            }
            Message::Fitted(scale) => {
                if scale.is_finite() && scale > 0.0 {
                    self.fit = scale;
                }
            }
            Message::Drew(mark) => {
                self.typing = false;
                let Some(picture) = self.ready() else { return };
                if let Some(shape) = model::from_mark(
                    &mark,
                    self.color,
                    picture.width as f32,
                    picture.height as f32,
                ) {
                    self.shapes.push(shape);
                    self.status = None;
                }
            }
            Message::Pointed(point) => {
                self.typing = false;
                if self.tool == Tool::Note {
                    self.add_note(point, cx);
                }
            }
            Message::Undo => {
                self.typing = false;
                self.undo();
            }
            Message::UndoKey => {
                if !self.typing {
                    self.undo();
                }
            }
            Message::NoteText(n, text) => {
                self.typing = true;
                self.set_note_text(n, text);
            }
            Message::Comment(text) => {
                self.typing = true;
                self.comment = logic::clip(&text, logic::COMMENT_MAX, true);
            }
            Message::Target(text) => {
                self.typing = true;
                self.target = logic::clip(&text, logic::PANE_MAX, false);
            }
            Message::Send => self.send(cx),
            Message::Exported(result) => self.exported(result, cx),
            Message::JsonWritten(result) => self.json_written(result, cx),
            Message::Told(result) => self.told(result),
        }
    }

    fn view(&self) -> Element<Message> {
        let t = &self.t;
        let ready = self.ready();
        let mut body: Vec<Element<Message>> = Vec::new();
        if let Some(notice) = &self.notice
            && !(ready.is_some() && notice.kind == AlertKind::Info)
        {
            body.push(self.notice_box(notice));
        }
        match ready {
            None => {
                let denied = self
                    .notice
                    .as_ref()
                    .is_some_and(|notice| notice.kind == AlertKind::Denied);
                if self.phase != Phase::Loading && !denied {
                    body.push(
                        empty(t.t("empty_title"))
                            .text(t.t("empty_text"))
                            .action(button(t.t("choose")).primary().on_press(Message::Choose))
                            .action(button(t.t("paste")).on_press(Message::Paste))
                            .into(),
                    );
                }
            }
            Some(picture) => {
                body.extend(self.toolbar());
                body.push(self.stage(picture));
                body.push(self.notes());
                body.push(self.fields());
            }
        }
        let mut parts: Vec<Element<Message>> = vec![
            self.header(),
            scroll(column(body).spacing(16.0).padding(12.0))
                .id("page")
                .into(),
        ];
        if ready.is_some() {
            parts.push(self.foot());
        }
        page(parts).fill().into()
    }

    fn subscriptions(&self) -> Subscriptions<Message> {
        Subscriptions::none()
            .tab_params(Message::Params)
            .theme(Message::Theme)
            .shortcut("mod+z", Message::UndoKey)
            .shortcut("mod+v", Message::Paste)
    }
}

export_kit!(Annotate);
