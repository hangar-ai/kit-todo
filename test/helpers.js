"use strict";
// Shared by the tests: the kit's folder, and a temporary one to build a tree of files in.
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const KIT = path.join(__dirname, "..");

function tempFolder(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "todo-kit-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

module.exports = { KIT, tempFolder };
