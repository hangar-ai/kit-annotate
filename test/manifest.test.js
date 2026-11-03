"use strict";
// The manifest, checked by hand against docs/native/kits-native.md in Hangar: Hangar's validator is
// Rust and cannot be reached from a Node test. A value's rule is written in what Rust and
// JavaScript read alike, as Hangar's window reads it. The tab itself is tested in native/tests/.
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const KIT = path.join(__dirname, "..");
const manifest = JSON.parse(fs.readFileSync(path.join(KIT, "hangar-kit.json"), "utf8"));
const exists = (relative) => fs.existsSync(path.join(KIT, relative));
const whole = (pattern) => new RegExp(`^(?:${pattern})$`, "u");

function safePath(relative, extensions, at) {
  assert.equal(typeof relative, "string", at);
  assert.doesNotMatch(relative, /^([/\\]|[A-Za-z]:)|:|(^|[/\\])\.\.([/\\]|$)/, `${at}: must stay inside the kit`);
  assert.ok(extensions.some((ext) => relative.toLowerCase().endsWith(ext)), `${at}: ${extensions.join("/")}`);
  assert.ok(exists(relative), `${at}: ${relative} must exist`);
}

test("the manifest's own fields", () => {
  assert.equal(manifest.schema, 2, "a native tab needs schema 2");
  assert.equal(manifest.id, "annotate");
  assert.equal(manifest.name, "Annotate");
  assert.match(manifest.version, /^\d+\.\d+\.\d+$/);
  assert.ok(manifest.description.trim());
  assert.equal(manifest.hangar, ">=2.0.0");
  assert.equal(manifest.homepage, "https://github.com/hangar-ai/kit-annotate");
  safePath(manifest.icon, [".svg"], "icon");
});

test("it carries its card for the library, and index.json is left to the release", () => {
  assert.ok(manifest.gallery.permissionsSummary.trim());
  assert.ok(manifest.gallery.tags.length && manifest.gallery.tags.every((tag) => typeof tag === "string" && tag.trim()));
});

test("the plugin is named like the kit, and holds a skill and nothing that runs by itself", () => {
  assert.equal(manifest.plugin, undefined, "default: the root, since .claude-plugin exists");
  const plugin = JSON.parse(fs.readFileSync(path.join(KIT, ".claude-plugin", "plugin.json"), "utf8"));
  assert.equal(plugin.name, manifest.id);
  assert.equal(plugin.version, manifest.version);
  for (const part of ["hooks", ".mcp.json", ".lsp.json", "monitors", "bin", "settings.json"]) assert.ok(!exists(part), part);
  for (const key of ["hooks", "mcpServers", "lspServers", "monitors", "settings", "binaries", "dependencies"]) assert.equal(plugin[key], undefined, key);
  assert.equal(manifest.commands, undefined, "no command either");
  assert.equal(manifest.setup, undefined);
});

test("the tab: three permissions and nothing more, one tab per image", () => {
  assert.equal(manifest.tabs.length, 1);
  const [tab] = manifest.tabs;
  assert.equal(tab.id, "annotate");
  assert.equal(tab.title, "Annotate");
  safePath(tab.entry, [".wasm"], "entry");
  safePath(tab.icon, [".svg"], "icon");
  assert.deepEqual([...tab.permissions].sort(), ["files.read", "files.write", "panes.send"]);
  assert.notEqual(tab.singleton, true, "one tab per image: another image is another tab");
});

test("the tab is native: its entry is a WebAssembly module of the kit, built from native/", () => {
  const [tab] = manifest.tabs;
  assert.equal(tab.entry, "tabs/annotate.wasm");
  assert.equal(tab.native, undefined, "no `native` field: the entry is the module");
  const head = fs.readFileSync(path.join(KIT, tab.entry)).subarray(0, 8);
  assert.deepEqual([...head], [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00], "a WebAssembly module, version 1");
  assert.ok(exists("native/Cargo.toml") && exists("native/Cargo.lock"), "the source it is built from, locked");
  assert.ok(exists("scripts/build-native.sh"));
  assert.ok(!exists("tabs/annotate"), "no page beside the module");
});

test("the params: only a path, and what its rule lets through", () => {
  const { params } = manifest.tabs[0];
  assert.deepEqual(Object.keys(params), ["path"]);
  assert.ok(params.path.description.trim());
  const rule = whole(params.path.pattern);
  for (const good of [".hangar/capture.png", "shots/a b.jpg", "sub\\shot.webp"]) assert.ok(rule.test(good), good);
  for (const bad of ["C:/x.png", "a:b", "a\nb", ""]) assert.ok(!rule.test(bad), JSON.stringify(bad));
  assert.doesNotMatch(params.path.pattern, /\(\?[<!=]/, "Rust's regex has no look-around");
});

test("the skill says what the brief says", () => {
  const skill = fs.readFileSync(path.join(KIT, "skills", "annotate", "SKILL.md"), "utf8").replace(/\r\n/g, "\n");
  assert.match(skill, /^---\nname: annotate\ndescription: .+\n---\n/);
  assert.match(skill, /kit_tab_open/);
  assert.match(skill, /"kit": "annotate"/);
  assert.match(skill, /"tab": "annotate"/);
  assert.match(skill, /Take the capture/);
  assert.match(skill, /Wait/);
  assert.match(skill, /Read the PNG/);
});

test("nothing personal, nothing outside its folder is named", () => {
  const files = [];
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      // `target`: the native tab's build, never committed (only tabs/annotate.wasm is).
      if ([".git", "node_modules", "target"].includes(entry.name)) continue;
      if (entry.isDirectory()) walk(path.join(dir, entry.name));
      else files.push(path.join(dir, entry.name));
    }
  };
  walk(KIT);
  const self = __filename;
  for (const file of files.filter((other) => other !== self)) {
    if (file.endsWith(".wasm")) continue;
    const text = fs.readFileSync(file, "utf8");
    assert.doesNotMatch(text, /[A-Za-z]:[\\/]Users[\\/]|\/home\/[a-z]/i, file);
  }
});
