# TODO

A Hangar kit that shows the debt the code already names, the `TODO`, `FIXME`, `HACK` and `XXX`
comments, and hands any of it to an agent in one click.

## What it does

**A tab**, « TODO », native: Rust compiled to WebAssembly, drawn by Hangar 2.0 or later with its
own widgets, in your theme. It scans the workspace when it opens (and when you press **Analyser**) and lists:

- the **markers**, grouped **by file** or **by tag**, with a filter chip per tag (with its count) and
  a search over the comment and the file name;
- for each one its tag, its comment, `file:line`, and two buttons: **Ouvrir**, which opens the file
  in Hangar's preview, and **Confier à un agent**.

The tab's badge counts the markers. **Qui l’a écrit** (per file) asks git who wrote each marker of
that file and when, and shows `author, 3 months ago` under it.

**Confier à un agent** sends one marker to a Claude Code agent of the workspace (the orchestrator by
default, or the one you pick). The brief names the file, the line and the comment, says it comes from
the kit's scan and not from the person, and puts the comment between markers as data to read, never
instructions to follow: anyone who can write to the repository can write a comment.

**What counts as a marker.** A comment that *starts* with the tag, in a language whose comments the
scan knows (`// TODO: …`, `# FIXME(ana) …`, `/* HACK … */`, `<!-- TODO -->`, a line inside a block
comment). The word in prose, in a string (`"// TODO"`), in a name, in lowercase, or `todo!()` in Rust,
is not one.

Designed states: scanning, nothing found, too many to list (the totals stay complete), nothing
matches the filters, command not allowed, scan failed, no agent to send to, a send or an opening
refused.

**A skill** that tells agents to read the last recorded scan rather than search the whole code again
when they are only asked what is left to do.

## What it runs

One declared command, `scan`: `node {kitDir}/tools/scan.js --blame={file}`, started in the workspace
folder. Hangar shows it to you verbatim when you enable the kit. Pure Node, no dependency. With an
empty `file` it prints one JSON document of the markers (`file`, `line`, `tag`, `text`); with a path
it lists that file's markers with their `author` and `at` (when) from `git blame`.

- **Read-only.** It opens files to read them and, for the author of a file, runs
  `git blame --line-porcelain -- <file>`. It writes nothing, installs nothing, calls no network and
  follows no link. A test fails if the script could write to disk.
- **What it reads.** It skips `.git`, `node_modules`, `target` and `dist` at any depth, and what the
  `.gitignore` files of the workspace list (patterns, negations, `**`, nested files). It reads only
  files of a known language, smaller than 1 MB, that are not binary, and lines shorter than 2 000
  characters.
- **Capped.** At most 1 000 markers are listed. Past that the totals keep counting (to 20 000) and
  the tab says the list is partial. A scan that takes more than 40 s stops and says so.
- **The tab** holds `command:scan` (to run that command), `panes.read` (to list the agents),
  `panes.send` (to hand a marker over, when you click), `preview.show` (to open a file, when you
  click), `data.read` and `data.write` (to keep the last scan in the kit's data folder,
  `last.json`). It writes no file of the project.

## What it does not do

- It does not fix anything: resolving a marker is the agent's job, with your approval of its work
  like any other change.
- It does not read languages it has no comment syntax for, nor JSON (no comments), nor `.env` files.
- Preview cannot open a hidden file or folder (`.github/…`): those rows have no **Ouvrir**.
- A marker on several lines of comment is listed by its first line.

## The native tab

The tab is **Rust compiled to WebAssembly**: `native/` is the crate, `tabs/todo.wasm` the module the
manifest's `entry` names (the manifest is schema 2, `"hangar": ">=2.0.0"`), and Hangar draws its page
with its own widgets, in your theme, with no browser engine. There is no HTML page. It holds the
six permissions listed above, and does the scan, keeps the last scan in the kit's data folder, filters
by tag, searches, groups, and offers **Qui l'a écrit**, **Ouvrir**, **Confier à un agent** and the
badge through them.

- **The last scan is read back.** The tab writes the document it reads (and the skill's: `schema`,
  `at`, `total`, `counts`, `truncated`, `incomplete`, `items`), so the stored scan comes back when the
  tab opens.
- **A scan too large to store is said so.** With 1 000 long paths and comments the document can pass
  the 1 MB `data.write` takes. The tab then shows the whole scan, stores the first markers that fit
  (`truncated`, with the complete totals) and says how many were kept; a refused write is said too.
- **An answer about an old scan is let go**: a send that finishes after a new scan does not mark
  another marker as sent.
- **An opening refused by `preview.show` says so**, rather than naming `panes.send`.

What a native tab cannot do (docs/native/kits-native.md in Hangar's repository):

- **Select text with the pointer.** A comment or a path cannot be selected; copy it from the file
  (**Ouvrir**).
- **Accessibility roles and labels** (`role=alert`, `aria-label`, the heading levels): the native
  toolkit has no accessibility tree yet; the **Ouvrir** buttons carry the file and line as a tooltip.
- **A client-side timeout.** Only the scan's own limit, 60 s, ends a scan (`timeoutSeconds` of the
  manifest).

The times follow Hangar's formats: "Scanned 5 min ago", then "today at 9:13 PM".

### Building it

`rustup target add wasm32-unknown-unknown` once, then:

```sh
scripts/build-native.sh        # native/ -> tabs/todo.wasm (release, wasm32-unknown-unknown)
cargo test --manifest-path native/Cargo.toml
scripts/check-native.sh        # the committed module is built from the committed sources
```

Hangar runs the `.wasm` a kit ships and never builds one, so `tabs/todo.wasm` is committed, in the
commit that changes `native/`. The build puts a hash of `native/Cargo.toml`, `Cargo.lock` and `src/`
in the module, and `check-native.sh` (no Rust needed, run by CI first) fails when it is not the
hash of the sources next to it. The SDK is `hangar-kit-sdk` from the `feat/native-rust` branch of
<https://github.com/hangar-ai/hangar>, pinned by `native/Cargo.lock` until a release tag holds it.
The paths of the machine that built it are replaced in the module, so it holds nothing personal.

To try it: link this folder in Settings → Kits of a native Hangar, open the tab, and after each
build reload the tab from its bar; or `cargo run -p hangar-kit-host --example preview --
tabs/todo.wasm` in Hangar's repository, a pretend Hangar that does not run commands (the tab then
shows its failed-scan state).

## Tests and fixtures

`node --test kits/todo/`. They run the script on a fixture tree built on the fly (comments in a dozen
languages, ignored folders, a `.gitignore` with negations, a binary file, strings and prose that must
not match, a real git repository for the author), and read the manifest and the kit's files.

`cargo test --manifest-path native/Cargo.toml` runs the native tab's: its states and calls through
the SDK's test harness (the same runtime the module runs, on the host, answered by a pretend Hangar),
and the manifest's schema 2 `entry`.

## Get it

Link `kits/todo` from Hangar, or clone
<https://github.com/hangar-ai/kit-todo>. Needs Hangar 2.0.0 or later (an older one says the kit
needs a newer Hangar) and Node.js on the PATH (and git, for the authors). The native tab needs
nothing more: the module is in the clone.

## This repository

`node --test` runs the tests of the scan script and of the manifest, with Node alone; `cargo test`
(in `native/`) the native tab's. These tests need nothing from Hangar's repository.

CI calls [kit-check](https://github.com/hangar-ai/.github/blob/rust-rewrite/.github/workflows/kit-check.yml),
and checks the native tab: the committed module is the sources', the tests pass, the module builds.
A release is a tag, `v` and the manifest's version, and it reaches Hangar's gallery once
[the catalog](https://github.com/hangar-ai/catalog) is updated and reviewed.
