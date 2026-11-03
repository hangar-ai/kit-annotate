//! The native tab and its manifest: the module the tab's `entry` names, and
//! what the tab calls against what it declares.

mod common;

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde_json::{Value, json};

use hangar_kit_sdk::api::{self, Needs};
use hangar_kit_sdk::testing::{Harness, StartExt};
use hangar_kit_sdk::ui::{Mark, Point};

use common::*;

fn kit() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn manifest() -> Value {
    let text = std::fs::read_to_string(kit().join("hangar-kit.json")).expect("the manifest");
    serde_json::from_str(&text).expect("JSON")
}

/// A path of the kit: relative, never out of it, of one of these
/// extensions, and there.
fn safe_path(relative: &str, extension: &str, at: &str) {
    let path = Path::new(relative);
    assert!(
        path.components()
            .all(|part| matches!(part, Component::Normal(_))),
        "{at}: {relative} must stay inside the kit"
    );
    assert!(
        !relative.contains(':') && !relative.contains('\\'),
        "{at}: {relative}"
    );
    assert!(
        relative.to_lowercase().ends_with(extension),
        "{at}: a {extension}"
    );
    assert!(kit().join(path).is_file(), "{at}: {relative} must exist");
}

#[test]
fn the_tab_is_native_and_its_entry_is_its_module() {
    let manifest = manifest();
    assert_eq!(manifest["schema"], 2, "a native tab needs schema 2");
    assert_eq!(manifest["hangar"], ">=2.0.0");
    let tab = &manifest["tabs"][0];
    assert_eq!(tab["entry"], "tabs/annotate.wasm");
    safe_path(tab["entry"].as_str().unwrap(), ".wasm", "entry");
    assert!(tab.get("native").is_none(), "no `native` field in schema 2");
    assert!(
        !kit().join("tabs/annotate").exists(),
        "no page beside the module"
    );
    let mut permissions: Vec<&str> = tab["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|permission| permission.as_str().unwrap())
        .collect();
    permissions.sort();
    assert_eq!(
        permissions, GRANTED,
        "the three permissions and nothing more"
    );
}

/// Reads an unsigned LEB128 at `at`, moving past it.
fn leb(bytes: &[u8], at: &mut usize) -> u64 {
    let (mut value, mut shift) = (0u64, 0);
    loop {
        let byte = bytes[*at];
        *at += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
        shift += 7;
    }
}

fn name(bytes: &[u8], at: &mut usize) -> String {
    let len = leb(bytes, at) as usize;
    let text = String::from_utf8(bytes[*at..*at + len].to_vec()).expect("a name");
    *at += len;
    text
}

fn limits(bytes: &[u8], at: &mut usize) {
    let flags = bytes[*at];
    *at += 1;
    leb(bytes, at);
    if flags & 1 == 1 {
        leb(bytes, at);
    }
}

/// The module's imports and exports, by name, from its sections.
fn interface(bytes: &[u8]) -> (BTreeSet<String>, BTreeSet<String>) {
    assert_eq!(
        &bytes[..8],
        b"\0asm\x01\0\0\0",
        "a WebAssembly module, version 1"
    );
    let (mut imports, mut exports) = (BTreeSet::new(), BTreeSet::new());
    let mut at = 8;
    while at < bytes.len() {
        let id = bytes[at];
        at += 1;
        let size = leb(bytes, &mut at) as usize;
        let end = at + size;
        let mut cursor = at;
        match id {
            2 => {
                for _ in 0..leb(bytes, &mut cursor) {
                    let module = name(bytes, &mut cursor);
                    let field = name(bytes, &mut cursor);
                    imports.insert(format!("{module}.{field}"));
                    let kind = bytes[cursor];
                    cursor += 1;
                    match kind {
                        0 => {
                            leb(bytes, &mut cursor);
                        }
                        1 => {
                            cursor += 1;
                            limits(bytes, &mut cursor);
                        }
                        2 => limits(bytes, &mut cursor),
                        3 => cursor += 2,
                        other => panic!("an import of kind {other}"),
                    }
                }
            }
            7 => {
                for _ in 0..leb(bytes, &mut cursor) {
                    exports.insert(name(bytes, &mut cursor));
                    cursor += 1;
                    leb(bytes, &mut cursor);
                }
            }
            _ => {}
        }
        at = end;
    }
    (imports, exports)
}

#[test]
fn the_module_imports_only_what_hangar_gives_and_exports_the_abi() {
    let bytes = std::fs::read(kit().join("tabs/annotate.wasm")).expect("tabs/annotate.wasm, built");
    assert!(bytes.len() < 4 * 1024 * 1024, "{} bytes", bytes.len());
    let (imports, exports) = interface(&bytes);
    for import in &imports {
        assert!(
            ["hangar.log", "hangar.now"].contains(&import.as_str()),
            "{import}: no WASI, no wasm-bindgen"
        );
    }
    for export in [
        "memory",
        "hangar_abi_version",
        "hangar_alloc",
        "hangar_init",
        "hangar_update",
        "hangar_view",
    ] {
        assert!(exports.contains(export), "{export}");
    }
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        !text.contains("/home/") && !text.contains("/Users/"),
        "no one's folder is written in it"
    );
}

/// A whole session, every call answered: what the tab ever asks of Hangar.
fn every_method() -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut answer = |tab: &mut Harness<Annotate>, method: &str, reply: Result<Value, _>| {
        let call = tab.reply_to(method, reply);
        seen.insert(call.method);
    };
    let mut tab = asking("a.png", start().caller("Ava"));
    answer(
        &mut tab,
        "image.size",
        Ok(json!({ "width": 800, "height": 600 })),
    );
    tab.draw("shot", Mark::arrow((0.0, 0.0), (90.0, 90.0)));
    tab.press("Note");
    tab.point("shot", Point::new(5.0, 5.0));
    tab.press("Send");
    answer(&mut tab, "canvas.export", exported(PNG, 1.0));
    answer(&mut tab, "files.write", Ok(json!({})));
    answer(
        &mut tab,
        "panes.send",
        Ok(json!({ "queued": true, "pane": "Ava" })),
    );
    tab.press("Another image");
    answer(&mut tab, "ui.confirm", Ok(json!(true)));
    tab.press("Choose an image");
    answer(
        &mut tab,
        "files.pick",
        Ok(json!({ "name": "b.png", "size": 9, "held": "held-1" })),
    );
    answer(
        &mut tab,
        "image.size",
        Ok(json!({ "width": 8, "height": 6 })),
    );
    tab.press("Another image");
    tab.shortcut("mod+v");
    answer(
        &mut tab,
        "ui.pasteImage",
        Ok(json!({ "held": "held-2", "width": 8, "height": 6 })),
    );
    assert!(tab.calls().is_empty());
    seen
}

#[test]
fn the_tab_calls_nothing_it_did_not_ask_for() {
    let seen = every_method();
    let expected: BTreeSet<String> = [
        "image.size",
        "canvas.export",
        "files.write",
        "panes.send",
        "ui.confirm",
        "files.pick",
        "ui.pasteImage",
    ]
    .map(String::from)
    .into();
    assert_eq!(seen, expected);
    let manifest = manifest();
    let declared: Vec<&str> = manifest["tabs"][0]["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|permission| permission.as_str().unwrap())
        .collect();
    for method in &seen {
        match api::needs(method) {
            Some(Needs::Nothing) => {}
            Some(Needs::Permission(permission)) => {
                assert!(
                    declared.contains(&permission),
                    "{method} needs {permission}"
                )
            }
            other => panic!("{method}: {other:?}"),
        }
    }
    // image.size needs what its source needs: a file of the workspace,
    // files.read. panes.read (to list the panes) would be one more.
    assert!(declared.contains(&"files.read"));
    assert!(!seen.contains("panes.list") && !seen.contains("workspace.info"));
}
