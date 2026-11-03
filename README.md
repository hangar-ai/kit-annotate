# Annotate

A Hangar kit. Describing a visual problem in words is slow. The agent shows a capture, or you
paste one; you circle what is wrong, write a word beside it, and the agent receives the
annotated image and your notes.

## What it does

- **Opens an image three ways**: from the `path` an agent gives, pasted with Ctrl+V, or dropped
  on the tab. A button picks a file too. PNG, JPEG, WebP and GIF; never an SVG or a PDF.
- **Four marks, nothing else**: an arrow, a rectangle, a freehand stroke, and a numbered note
  whose word you write in the list under the image. Five colours, undo, and a zoom that fits the
  image to the panel by default. It is a canvas and no dependency.
- **« Envoyer » (Send) writes two files** in the workspace: the annotated PNG at
  `.hangar/annotations/<date>-<time>.png`, and `<date>-<time>.json` beside it, with the comment,
  the numbered notes and a line per mark. It never replaces a file: a taken name gets `-2`.
  An image too heavy for 10 MB is written at a smaller size, and the message says so.
- **Then it tells the agent** with one message: the path of the PNG (so the agent can read it),
  how many marks of each kind, your comment, and each note with its position in the image's
  pixels. It goes to the agent that opened the tab; otherwise to the pane you name. With no
  pane, the button reads « Enregistrer » (Save): the files are saved and the path shown.
- **English and French**, in Hangar's language; light and dark, in Hangar's theme; legible at the
  side panel's narrowest.

## A native tab

The tab is native, in Rust built to WebAssembly: [`native/`](native) builds
[`tabs/annotate.wasm`](tabs/annotate.wasm), which the manifest names as the tab's `entry`. It needs
Hangar 2.0 or later, which runs the module and draws it with its own widgets.

How it works:

- **The capture is never read into the tab.** Hangar shows it where it is, reads its size
  (`image.size`, under `files.read`), and draws the annotated PNG itself (`canvas.export`, a
  `files.write` that never replaces a file), at three quarters, a half, 35 % or a quarter when the
  full size would pass 10 MB.
- **An image from elsewhere** comes from the file chooser (« Choose an image », under `files.read`)
  or the clipboard (Ctrl+V, or « Paste an image »). A picture from outside the workspace is held by
  the tab, up to 10 MB, until it closes; only the annotated PNG is written.
- **Ctrl+Z** undoes the last mark, except right after typing in a field, until a press on the
  canvas or a button: the native fields have no undo of their own, so the keystroke reaches the tab.

What it cannot do: **take a file dropped on the tab** (the window hears a drop, not where it
lands; the chooser and the paste stand in), and, with `files.read` refused, choose a file (the
chooser needs it; pasting still works).

## What it can touch

Three permissions, shown to you before you enable the kit. It runs nothing by itself: no hook,
no MCP server, no command.

| Permission    | What for                                                                                  |
| ------------- | ----------------------------------------------------------------------------------------- |
| `files.read`  | Read the image an agent named by its `path`.                                              |
| `files.write` | Write the annotated PNG and its JSON under `.hangar/annotations/`, never over a file.     |
| `panes.send`  | Tell the agent that asked, or the pane you named, where the image is.                     |

Hangar words `files.write` as changing the workspace's text files. This kit also makes new images
with it (`files.writeBinary`, natively `canvas.export`): PNG, JPEG or WebP only, whose first bytes
are checked, never replacing a file, 10 MB at most. Nothing under `.git` can be reached, and nothing
leaves the machine.

**What reaches the agent:** the image's path, counts, your comment and your notes, as you wrote
them, in sentences fixed in [`native/src/answer.rs`](native/src/answer.rs).

What it cannot do: it cannot list the workspace's panes (that needs `panes.read`, one more
permission than it asks for), so without a caller you type the pane's name. Drawing is by pointer;
by keyboard you can write the comment and the notes' words but not place a mark.

## Install

You need Hangar 2.0.0 or later, which runs native tabs.

1. In Hangar, Settings › Kits, add the GitHub link:
   `https://github.com/hangar-ai/kit-annotate`
   (or link a local copy of this folder).
2. Enable it on a workspace, and read what you accept: the three permissions above.
3. Restart the workspace's Claude Code panes, so they load the skill.

## For agents

The skill in [`skills/annotate/SKILL.md`](skills/annotate/SKILL.md) says it: when you need the
person's eye on something visual, take the capture inside the workspace folder, open it with
`kit_tab_open`, then wait for the answer and read the PNG it names.

```json
{ "kit": "annotate", "tab": "annotate", "params": { "path": ".hangar/capture.png" } }
```

| Param  | What                                                                                 |
| ------ | ------------------------------------------------------------------------------------ |
| `path` | The image, relative to the workspace folder. Without it, the tab waits for a paste.  |

## How it is built

```
annotate/
├── hangar-kit.json            the kit: schema 2, id, hangar >=2.0.0, the tab, its param and permissions
├── icon.svg
├── .claude-plugin/plugin.json the Claude Code plugin (name = the kit's id)
├── skills/annotate/SKILL.md
├── tabs/annotate.wasm         the tab, built from native/ and committed: the manifest's entry
├── native/                    the tab's crate (hangar-kit-sdk, from git)
│   ├── src/logic.rs           the words in en and fr, file names, zoom
│   ├── src/model.rs           the marks, and the canvas marks they are drawn as
│   ├── src/answer.rs          the message the agent gets, and the JSON beside the image
│   ├── src/lib.rs             the Kit: its update and its view
│   └── tests/                 the tab's cases, on the SDK's harness
├── scripts/build-native.sh    builds native/ into tabs/annotate.wasm (--check: fails if stale)
├── rust-toolchain.toml        the Rust that builds it, pinned so a rebuild gives the same bytes
├── test/manifest.test.js      node --test: the manifest, the plugin and the skill
└── package.json, run-tests.js let `node --test kits/annotate/` find the tests on any Node
```

The only colours that are not Hangar's are the five the person picks from to mark the image, in
`native/src/model.rs`.

## Tests

```
cargo test --manifest-path native/Cargo.toml
node --test kits/annotate/
scripts/build-native.sh --check   # the committed module is the one native/ builds
```

The native tab's tests run on the SDK's harness (Hangar's answers given by hand): what opens, what
is drawn, what is written and sent, and the module's own checks: it imports only what Hangar gives,
exports the ABI, and calls only what the three permissions allow. No test writes a real file. The
Node test reads the manifest, the plugin and the skill, with Node alone and no install step.

```
scripts/build-native.sh        # after a change to native/: rebuild, and commit tabs/annotate.wasm
```

The build needs the Rust that `rust-toolchain.toml` names and its `wasm32-unknown-unknown` target,
which rustup installs on the first run.

## This repository

CI calls [kit-check](https://github.com/hangar-ai/.github/blob/main/.github/workflows/kit-check.yml),
runs the native tab's tests, and builds the module again: it fails when the committed
`tabs/annotate.wasm` is not what the source builds. A release is a tag, `v` and the manifest's
version, and it reaches Hangar's gallery once [the catalog](https://github.com/hangar-ai/catalog)
is updated and reviewed.
