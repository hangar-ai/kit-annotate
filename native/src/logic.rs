//! What the tab says and the small rules around it: its words in English and
//! French, the params, the names of the files it writes, the zoom. No widget,
//! no call.

use hangar_kit_sdk::format::LocalTime;
use hangar_kit_sdk::i18n::Strings;

/// Where the image and its notes are written, relative to the workspace folder.
pub const FOLDER: &str = ".hangar/annotations";
/// The largest image file the person may choose. Hangar holds a picture from
/// outside the workspace up to 10 MB itself.
pub const INPUT_MAX_BYTES: u64 = 30 * 1024 * 1024;
/// What the file chooser offers: bitmaps, never an SVG (script) or a PDF.
pub const PICKABLE: [&str; 5] = ["png", "jpg", "jpeg", "webp", "gif"];
/// The scales Hangar tries an image at when its PNG is too heavy, until it
/// fits (`canvas.export` does it, and says which in `scale`).
pub const EXPORT_SCALES: [f32; 5] = [1.0, 0.75, 0.5, 0.35, 0.25];
pub const ZOOM_STEPS: [f32; 12] = [
    0.1, 0.15, 0.25, 0.33, 0.5, 0.67, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0,
];
pub const ZOOM_MIN: f32 = 0.1;
pub const ZOOM_MAX: f32 = 4.0;
pub const NOTE_MAX: usize = 500;
pub const COMMENT_MAX: usize = 2000;
pub const PANE_MAX: usize = 200;
/// A path param longer than this is no path.
pub const PATH_MAX: usize = 1024;

/// The tab's words in English. The page's, with what a native tab does
/// differently: no drop (the window hears a drop, not where it lands), the
/// file chooser under `files.read`, and a paste button.
pub const EN: &[(&str, &str)] = &[
    ("tab", "Annotate"),
    ("tools", "Tools"),
    ("tool_arrow", "Arrow"),
    ("tool_rect", "Rectangle"),
    ("tool_free", "Freehand"),
    ("tool_note", "Note"),
    ("tip_arrow", "Drag on the image to point at something."),
    ("tip_rect", "Drag on the image to frame something."),
    ("tip_free", "Draw on the image with the pointer."),
    ("tip_note", "Click where the problem is, then write a word."),
    ("color", "Color"),
    ("color_red", "Red"),
    ("color_orange", "Orange"),
    ("color_green", "Green"),
    ("color_blue", "Blue"),
    ("color_magenta", "Magenta"),
    ("undo", "Undo"),
    ("zoom", "Zoom"),
    ("zoom_in", "Zoom in"),
    ("zoom_out", "Zoom out"),
    ("zoom_fit", "Fit"),
    ("zoom_value", "{n} %"),
    ("canvas", "The image to annotate"),
    ("image_size", "{w} × {h} px"),
    ("another", "Another image"),
    ("another_ask", "Replace this image and drop its marks?"),
    ("another_yes", "Replace"),
    ("another_no", "Keep"),
    ("empty_title", "No image yet"),
    (
        "empty_text",
        "Paste a capture with Ctrl+V, or choose a file.",
    ),
    ("choose", "Choose an image"),
    ("paste", "Paste an image"),
    ("loading", "Reading {path}…"),
    ("denied_title", "This tab may not read files"),
    (
        "denied_text",
        "Accept files.read in Settings › Kits to open an image by its path or choose one. You can still paste one with Ctrl+V.",
    ),
    ("not_found", "{path} was not found in the workspace folder."),
    (
        "unsupported",
        "{path} is not a PNG, JPEG, WebP or GIF image.",
    ),
    ("read_failed", "Could not read {path}: {reason}"),
    (
        "bad_image",
        "This is not an image the tab can open. Use a PNG, JPEG, WebP or GIF.",
    ),
    ("too_large", "This image is larger than 30 MB."),
    (
        "paste_none",
        "The clipboard holds no image. Copy a capture, then paste it here.",
    ),
    ("paste_failed", "Could not paste the image: {reason}"),
    ("pick_failed", "Could not open the file chooser: {reason}"),
    (
        "keep_marks",
        "Marks are on this image. Use « Another image » to replace it.",
    ),
    ("notes_title", "Notes"),
    ("note_label", "Note {n}"),
    ("note_placeholder", "What is wrong here?"),
    ("no_notes", "Notes you place on the image are listed here."),
    ("comment_label", "Comment (optional)"),
    ("comment_placeholder", "What should the agent do?"),
    ("target_label", "Send to"),
    ("target_placeholder", "Pane name"),
    ("target_caller", "{pane} asked for this image."),
    (
        "target_none",
        "Without a pane name, the files are only saved.",
    ),
    ("send", "Send"),
    ("save", "Save"),
    ("sending", "Sending…"),
    ("sent", "Sent to {pane}: {png}"),
    ("saved", "Saved: {png}"),
    (
        "write_denied",
        "This tab may not write files. Accept files.write in Settings › Kits.",
    ),
    (
        "write_unknown",
        "This Hangar cannot save images yet. Update Hangar, then try again.",
    ),
    ("write_failed", "Could not save the image: {reason}"),
    (
        "json_failed",
        "The image is saved, but not its notes file: {reason}",
    ),
    (
        "send_denied",
        "Saved: {png}. This tab may not tell an agent: accept panes.send in Settings › Kits.",
    ),
    (
        "send_not_found",
        "Saved: {png}. No pane is named {pane} in this workspace.",
    ),
    (
        "send_unavailable",
        "Saved: {png}. {pane} cannot take a message right now.",
    ),
    ("send_failed", "Saved: {png}. {pane} was not told: {reason}"),
    ("too_heavy", "The image is too heavy to save, even reduced."),
    ("retry", "Try again"),
];

pub const FR: &[(&str, &str)] = &[
    ("tab", "Annoter"),
    ("tools", "Outils"),
    ("tool_arrow", "Flèche"),
    ("tool_rect", "Rectangle"),
    ("tool_free", "Main levée"),
    ("tool_note", "Note"),
    (
        "tip_arrow",
        "Faites glisser sur l’image pour pointer quelque chose.",
    ),
    (
        "tip_rect",
        "Faites glisser sur l’image pour entourer quelque chose.",
    ),
    ("tip_free", "Dessinez sur l’image avec le pointeur."),
    (
        "tip_note",
        "Cliquez là où est le problème, puis écrivez un mot.",
    ),
    ("color", "Couleur"),
    ("color_red", "Rouge"),
    ("color_orange", "Orange"),
    ("color_green", "Vert"),
    ("color_blue", "Bleu"),
    ("color_magenta", "Magenta"),
    ("undo", "Annuler"),
    ("zoom", "Zoom"),
    ("zoom_in", "Zoom avant"),
    ("zoom_out", "Zoom arrière"),
    ("zoom_fit", "Ajuster"),
    ("zoom_value", "{n} %"),
    ("canvas", "L’image à annoter"),
    ("image_size", "{w} × {h} px"),
    ("another", "Autre image"),
    (
        "another_ask",
        "Remplacer cette image et perdre ses marques ?",
    ),
    ("another_yes", "Remplacer"),
    ("another_no", "Garder"),
    ("empty_title", "Pas encore d’image"),
    (
        "empty_text",
        "Collez une capture avec Ctrl+V ou choisissez un fichier.",
    ),
    ("choose", "Choisir une image"),
    ("paste", "Coller une image"),
    ("loading", "Lecture de {path}…"),
    ("denied_title", "Cet onglet ne peut pas lire de fichiers"),
    (
        "denied_text",
        "Acceptez files.read dans Réglages › Kits pour ouvrir une image par son chemin ou en choisir une. Vous pouvez toujours en coller une avec Ctrl+V.",
    ),
    (
        "not_found",
        "{path} est introuvable dans le dossier du workspace.",
    ),
    (
        "unsupported",
        "{path} n’est pas une image PNG, JPEG, WebP ou GIF.",
    ),
    ("read_failed", "Impossible de lire {path} : {reason}"),
    (
        "bad_image",
        "Cette image ne peut pas être ouverte. Utilisez un PNG, un JPEG, un WebP ou un GIF.",
    ),
    ("too_large", "Cette image dépasse 30 Mo."),
    (
        "paste_none",
        "Le presse-papiers ne contient pas d’image. Copiez une capture, puis collez-la ici.",
    ),
    ("paste_failed", "Impossible de coller l’image : {reason}"),
    (
        "pick_failed",
        "Impossible d’ouvrir le sélecteur de fichiers : {reason}",
    ),
    (
        "keep_marks",
        "Des marques sont posées sur cette image. Utilisez « Autre image » pour la remplacer.",
    ),
    ("notes_title", "Notes"),
    ("note_label", "Note {n}"),
    ("note_placeholder", "Qu’est-ce qui ne va pas ici ?"),
    ("no_notes", "Les notes posées sur l’image sont listées ici."),
    ("comment_label", "Commentaire (facultatif)"),
    ("comment_placeholder", "Que doit faire l’agent ?"),
    ("target_label", "Envoyer à"),
    ("target_placeholder", "Nom du pane"),
    ("target_caller", "{pane} a demandé cette image."),
    (
        "target_none",
        "Sans nom de pane, les fichiers sont seulement enregistrés.",
    ),
    ("send", "Envoyer"),
    ("save", "Enregistrer"),
    ("sending", "Envoi…"),
    ("sent", "Envoyé à {pane} : {png}"),
    ("saved", "Enregistré : {png}"),
    (
        "write_denied",
        "Cet onglet ne peut pas écrire de fichiers. Acceptez files.write dans Réglages › Kits.",
    ),
    (
        "write_unknown",
        "Cette version de Hangar ne sait pas encore enregistrer d’images. Mettez Hangar à jour, puis réessayez.",
    ),
    (
        "write_failed",
        "Impossible d’enregistrer l’image : {reason}",
    ),
    (
        "json_failed",
        "L’image est enregistrée, pas son fichier de notes : {reason}",
    ),
    (
        "send_denied",
        "Enregistré : {png}. Cet onglet ne peut pas prévenir un agent : acceptez panes.send dans Réglages › Kits.",
    ),
    (
        "send_not_found",
        "Enregistré : {png}. Aucun pane ne s’appelle {pane} dans ce workspace.",
    ),
    (
        "send_unavailable",
        "Enregistré : {png}. {pane} ne peut pas recevoir de message pour l’instant.",
    ),
    (
        "send_failed",
        "Enregistré : {png}. {pane} n’a pas été prévenu : {reason}",
    ),
    (
        "too_heavy",
        "L’image est trop lourde pour être enregistrée, même réduite.",
    ),
    ("retry", "Réessayer"),
];

/// "fr", "fr-FR" → "fr"; anything else is English.
pub fn pick_lang(locale: &str) -> &'static str {
    let locale = locale.trim().to_ascii_lowercase();
    match locale.strip_prefix("fr") {
        Some(rest) if rest.is_empty() || rest.starts_with(['-', '_']) => "fr",
        _ => "en",
    }
}

/// The tab's strings in the person's language, English filling in.
pub fn strings(locale: &str) -> Strings {
    Strings::new(pick_lang(locale)).add("en", EN).add("fr", FR)
}

/// The `path` param: a non-empty string of a sane length, else none. The host
/// checks the rest.
pub fn clean_path(value: Option<&str>) -> Option<String> {
    let path = value?.trim();
    (!path.is_empty() && path.chars().count() <= PATH_MAX).then(|| path.to_owned())
}

/// The last part of a path, by either slash.
pub fn base_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

/// `20261031-231512`, from local time: the name a pair of files shares.
pub fn stamp(at: &LocalTime) -> String {
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        at.year, at.month, at.day, at.hour, at.minute, at.second
    )
}

/// The names tried for one send: `stamp`, then `stamp-2`… when a file is
/// already there.
pub fn names(base: &str, tries: usize) -> Vec<String> {
    (0..tries)
        .map(|i| {
            if i == 0 {
                base.to_owned()
            } else {
                format!("{base}-{}", i + 1)
            }
        })
        .collect()
}

/// `2026-10-31T23:15:12.000Z`: what `Date.toISOString` writes, from
/// milliseconds since the epoch.
pub fn iso(at_ms: i64) -> String {
    let at = hangar_kit_sdk::format::local_time(at_ms, 0);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        at.year,
        at.month,
        at.day,
        at.hour,
        at.minute,
        at.second,
        at_ms.rem_euclid(1000)
    )
}

/// A zoom asked for, kept within what the tab offers.
pub fn clamp_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() {
        zoom.clamp(ZOOM_MIN, ZOOM_MAX)
    } else {
        1.0
    }
}

/// The next step above (`direction` > 0) or below `current` among
/// [`ZOOM_STEPS`], or the last one at the end.
pub fn step_zoom(current: f32, direction: i32) -> f32 {
    let eps = 1e-6;
    if direction > 0 {
        ZOOM_STEPS
            .into_iter()
            .find(|step| *step > current + eps)
            .unwrap_or(ZOOM_STEPS[ZOOM_STEPS.len() - 1])
    } else {
        ZOOM_STEPS
            .into_iter()
            .rev()
            .find(|step| *step < current - eps)
            .unwrap_or(ZOOM_STEPS[0])
    }
}

/// At most `max` characters, counted in code points; without `keep_breaks`,
/// every run of white space holding a line break becomes one space.
pub fn clip(text: &str, max: usize, keep_breaks: bool) -> String {
    let flat = if keep_breaks {
        text.to_owned()
    } else {
        flatten(text)
    };
    if flat.chars().count() > max {
        flat.chars().take(max).collect()
    } else {
        flat
    }
}

fn flatten(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if run.contains(['\r', '\n']) {
            out.push(' ');
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for c in text.chars() {
        if c.is_whitespace() {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}
