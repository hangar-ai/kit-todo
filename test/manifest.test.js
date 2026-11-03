"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { KIT } = require("./helpers");

const manifest = JSON.parse(fs.readFileSync(path.join(KIT, "hangar-kit.json"), "utf8"));
const read = (...parts) => fs.readFileSync(path.join(KIT, ...parts), "utf8");

function allFiles(dir = KIT, found = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if ([".git", ".hangar-ui", "node_modules", "target"].includes(entry.name)) continue;
    if (entry.isDirectory()) allFiles(path.join(dir, entry.name), found);
    else found.push(path.join(dir, entry.name));
  }
  return found;
}

test("the manifest's own fields", () => {
  assert.equal(manifest.schema, 2);
  assert.equal(manifest.id, "todo");
  assert.match(manifest.version, /^\d+\.\d+\.\d+$/);
  assert.ok(manifest.name.trim() && manifest.description.trim());
  assert.equal(manifest.homepage, "https://github.com/hangar-ai/kit-todo");
  assert.equal(manifest.hangar, ">=2.0.0");
  assert.ok(fs.existsSync(path.join(KIT, manifest.icon)));
  assert.equal(manifest.setup, undefined);
});

test("the gallery's card: one sentence, and its tags", () => {
  const { permissionsSummary, tags } = manifest.gallery;
  assert.ok(permissionsSummary.trim());
  assert.match(permissionsSummary, /command/);
  assert.match(permissionsSummary, /click/);
  assert.ok(Array.isArray(tags) && tags.length > 0 && tags.every((tag) => typeof tag === "string" && tag.trim()));
});

test("the plugin is found at the root, named like the kit, with its skill", () => {
  assert.equal(manifest.plugin, undefined);
  const plugin = JSON.parse(read(".claude-plugin", "plugin.json"));
  assert.equal(plugin.name, manifest.id);
  assert.equal(plugin.version, manifest.version);
  assert.match(read("skills", "todo", "SKILL.md").replace(/\r\n/g, "\n"), /^---\nname: todo\ndescription: .+\n---\n/);
});

test("one declared command: node on the kit's own script, one value that may be empty, and a rule that lets it be", () => {
  assert.deepEqual(Object.keys(manifest.commands), ["scan"]);
  const scan = manifest.commands.scan;
  assert.deepEqual(scan.run, ["node", "{kitDir}/tools/scan.js", "--blame={file}"]);
  assert.deepEqual(Object.keys(scan.args), ["file"]);
  assert.ok(scan.description.trim());
  assert.ok(Number.isInteger(scan.timeoutSeconds) && scan.timeoutSeconds >= 1 && scan.timeoutSeconds <= 120);
  assert.ok(fs.existsSync(path.join(KIT, "tools", "scan.js")));
  // Hangar anchors the rule at both ends: an empty value (scan everything) and a path must match, a line break must not.
  const rule = new RegExp(`^(?:${scan.args.file.pattern})$`);
  for (const ok of ["", "src/app.js", "a b/c.rs", "./-odd.js"]) assert.ok(rule.test(ok), ok);
  for (const bad of ["a\nb", "a\r"]) assert.ok(!rule.test(bad), JSON.stringify(bad));
});

test("the tab: « TODO », and exactly the six permissions it needs", () => {
  assert.equal(manifest.tabs.length, 1);
  const [tab] = manifest.tabs;
  assert.match(tab.id, /^[a-z0-9-]{1,40}$/);
  assert.equal(tab.title, "TODO");
  assert.deepEqual([...tab.permissions].sort(), ["command:scan", "data.read", "data.write", "panes.read", "panes.send", "preview.show"]);
  assert.ok(fs.existsSync(path.join(KIT, tab.icon)));
});

test("the tab is native: the entry is its module, with no `native` field and no page", () => {
  const [tab] = manifest.tabs;
  assert.equal(tab.entry, "tabs/todo.wasm");
  assert.equal(tab.native, undefined);
  const file = path.join(KIT, tab.entry);
  assert.ok(fs.statSync(file).size > 0 && fs.statSync(file).size <= 64 * 1024 * 1024);
  assert.ok(!fs.existsSync(path.join(KIT, "tabs", "todo")), "no HTML page is left");
});

test("nothing personal anywhere in the kit", () => {
  const forbidden = /C:\\Users|\/Users\/|\/home\/\w|@[\w-]+\.(com|fr)|[Cc]o-[Aa]uthored/;
  for (const file of allFiles()) {
    if (file.includes(`${path.sep}fixtures${path.sep}`)) continue;
    assert.doesNotMatch(fs.readFileSync(file, "utf8"), forbidden, path.relative(KIT, file));
  }
});

test("the README says what it does and what it runs", () => {
  const readme = read("README.md");
  assert.match(readme, /## What it does/);
  assert.match(readme, /## What it runs/);
  for (const command of ["node {kitDir}/tools/scan.js", "git blame --line-porcelain", "command:scan", "preview.show", "panes.send"]) assert.ok(readme.includes(command), command);
});
