//! Shared by the tests: a pretend Hangar answering the tab's calls as
//! Hangar does (an in-memory data folder, panes, the scan command, the
//! permissions the tab holds), the documents the
//! scan script prints, and a few ways to read the page.
#![allow(dead_code)]

use std::collections::BTreeMap;

use serde_json::{Value, json};

use hangar_kit_sdk::format;
use hangar_kit_sdk::testing::{self, Harness, StartExt};
use hangar_kit_sdk::ui::abi::Call;
use hangar_kit_sdk::ui::{Element, EventValue, HandlerId};
use hangar_kit_sdk::{Kit, api::KitError};
use kit_todo_tab::Todo;

pub type Answer = Result<Value, KitError>;

/// What the person gave the tab: every permission of the manifest.
pub const GRANTED: [&str; 6] = [
    "command:scan",
    "panes.read",
    "panes.send",
    "preview.show",
    "data.read",
    "data.write",
];

/// When the fixtures' scans were made: 2026-10-31T17:02:00Z.
pub fn at() -> i64 {
    format::parse_iso("2026-10-31T17:02:00Z").unwrap()
}

pub const MINUTE: i64 = 60_000;

pub fn denied(method: &str) -> KitError {
    KitError::new(
        KitError::DENIED,
        format!("this tab was not granted what {method} needs"),
    )
}

pub fn failure(code: &str) -> KitError {
    KitError::new(code, code)
}

/// A marker as the script prints it.
pub fn item(extra: Value) -> Value {
    let mut item =
        json!({ "file": "src/app.js", "line": 3, "tag": "TODO", "text": "wire the settings" });
    merge(&mut item, extra);
    item
}

/// A scan document of schema 1, its counts taken from its items.
pub fn doc(items: Vec<Value>, extra: Value) -> Value {
    let mut counts = json!({ "TODO": 0, "FIXME": 0, "HACK": 0, "XXX": 0 });
    for item in &items {
        let tag = item["tag"].as_str().unwrap().to_owned();
        counts[&tag] = json!(counts[&tag].as_i64().unwrap() + 1);
    }
    let mut document = json!({
        "schema": 1, "at": at(), "total": items.len(), "counts": counts,
        "truncated": false, "incomplete": false, "items": items,
    });
    merge(&mut document, extra);
    document
}

/// Four markers in three visible files and one hidden.
pub fn mixed() -> Value {
    doc(
        vec![
            item(json!({})),
            item(json!({ "line": 9, "tag": "FIXME", "text": "breaks with an empty list" })),
            item(
                json!({ "file": "src/lib.rs", "line": 1, "tag": "HACK", "text": "temporary index" }),
            ),
            item(
                json!({ "file": ".github/ci.yml", "line": 4, "tag": "XXX", "text": "pin the digest" }),
            ),
        ],
        json!({}),
    )
}

pub fn claude(name: &str, extra: Value) -> Value {
    let mut pane = json!({ "name": name, "agent": "claude", "role": "worker", "state": "idle", "canSend": true });
    merge(&mut pane, extra);
    pane
}

fn merge(into: &mut Value, extra: Value) {
    if let (Some(into), Value::Object(extra)) = (into.as_object_mut(), extra) {
        into.extend(extra);
    }
}

/// What `command.run` answers when scan.js printed `document`.
pub fn printed(document: Value, extra: Value) -> Answer {
    let mut answer = json!({
        "exitCode": 0, "stdout": format!("{document}\n"), "stderr": "", "timedOut": false, "truncated": false,
    });
    merge(&mut answer, extra);
    Ok(answer)
}

/// How the pretend Hangar answers.
pub struct Options {
    /// The kit's data folder.
    pub files: BTreeMap<String, String>,
    pub panes: Value,
    /// What `command.run` answers, given the `file` it was asked for (empty
    /// for the whole workspace).
    pub command: Box<dyn FnMut(&str) -> Answer>,
    /// The methods the tab may not call.
    pub deny: Vec<&'static str>,
    pub send_error: Option<&'static str>,
    pub open_error: Option<&'static str>,
    /// The methods whose calls stay unanswered, for the test to answer.
    pub hold: Vec<&'static str>,
    pub locale: &'static str,
    pub utc_offset: i32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            files: BTreeMap::new(),
            panes: json!([]),
            command: Box::new(|_| printed(doc(vec![], json!({})), json!({}))),
            deny: Vec::new(),
            send_error: None,
            open_error: None,
            hold: Vec::new(),
            locale: "en",
            utc_offset: 0,
        }
    }
}

impl Options {
    /// The command prints the same scan for the workspace, and for a file.
    pub fn scan(document: Value) -> Self {
        Options {
            command: Box::new(move |_| printed(document.clone(), json!({}))),
            ..Options::default()
        }
    }

    pub fn stored(mut self, document: &Value) -> Self {
        self.files
            .insert("last.json".to_owned(), document.to_string());
        self
    }
}

/// The tab, and the Hangar answering it.
pub struct Tab {
    pub tab: Harness<Todo>,
    pub files: BTreeMap<String, String>,
    pub panes: Value,
    command: Box<dyn FnMut(&str) -> Answer>,
    deny: Vec<&'static str>,
    send_error: Option<&'static str>,
    open_error: Option<&'static str>,
    hold: Vec<&'static str>,
    /// Every call the tab made, in order, answered or not.
    pub calls: Vec<Call>,
}

/// Starts the tab and answers what it asks until it waits for the test.
pub fn open(options: Options) -> Tab {
    let start = testing::start()
        .grant(GRANTED)
        .locale(options.locale)
        .utc_offset(options.utc_offset);
    let mut tab = Tab {
        tab: Harness::<Todo>::new(start),
        files: options.files,
        panes: options.panes,
        command: options.command,
        deny: options.deny,
        send_error: options.send_error,
        open_error: options.open_error,
        hold: options.hold,
        calls: Vec::new(),
    };
    tab.settle();
    tab
}

impl Tab {
    /// Answers every call waiting that is not held, and the calls those
    /// answers lead to.
    pub fn settle(&mut self) {
        loop {
            for call in self.tab.calls() {
                if !self.calls.iter().any(|seen| seen.id == call.id) {
                    self.calls.push(call.clone());
                }
            }
            let Some(call) = self
                .tab
                .calls()
                .iter()
                .find(|call| !self.hold.contains(&call.method.as_str()))
                .cloned()
            else {
                return;
            };
            let answer = self.answer(&call);
            self.tab.reply(call.id, answer);
        }
    }

    fn answer(&mut self, call: &Call) -> Answer {
        let method = call.method.as_str();
        let args = &call.params;
        if self.deny.contains(&method) {
            return Err(denied(method));
        }
        match method {
            "tab.setTitle" | "tab.setBadge" => Ok(json!({})),
            "data.read" => {
                let path = args["path"].as_str().unwrap_or_default();
                match self.files.get(path) {
                    Some(text) => Ok(json!({ "text": text })),
                    None => Err(failure("not_found")),
                }
            }
            "data.write" => {
                self.files.insert(
                    args["path"].as_str().unwrap_or_default().to_owned(),
                    args["text"].as_str().unwrap_or_default().to_owned(),
                );
                Ok(json!({}))
            }
            "panes.list" => Ok(self.panes.clone()),
            "panes.send" => match self.send_error {
                Some(code) => Err(KitError::new(code, format!("could not: {code}"))),
                None => Ok(json!({ "queued": true, "pane": args["pane"] })),
            },
            "command.run" => {
                let file = args["args"]["file"].as_str().unwrap_or_default().to_owned();
                (self.command)(&file)
            }
            "preview.show" => match self.open_error {
                Some(code) => Err(KitError::new(code, format!("could not: {code}"))),
                None => Ok(json!({ "opened": true, "path": args["path"] })),
            },
            _ => Err(denied(method)),
        }
    }

    /// The params of every call to `method`, in order.
    pub fn calls_to(&self, method: &str) -> Vec<Value> {
        self.calls
            .iter()
            .filter(|call| call.method == method)
            .map(|call| call.params.clone())
            .collect()
    }

    /// Answers the oldest call to `method` the test held.
    pub fn answer_held(&mut self, method: &str, answer: Answer) {
        self.tab.reply_to(method, answer);
        self.settle();
    }

    pub fn press(&mut self, label: &str) {
        self.tab.press(label);
        self.settle();
    }

    /// Presses the `n`th button labelled `label`, in the order they are drawn.
    pub fn press_nth(&mut self, label: &str, n: usize) {
        let handler = buttons(&self.tab, label)
            .get(n)
            .copied()
            .unwrap_or_else(|| panic!("no button number {n} labelled {label:?}"))
            .unwrap_or_else(|| panic!("button number {n} labelled {label:?} is disabled"));
        self.tab.ui(handler, None);
        self.settle();
    }

    pub fn type_into(&mut self, name: &str, text: &str) {
        self.tab.type_into(name, text);
        self.settle();
    }

    pub fn choose(&mut self, name: &str, value: &str) {
        self.tab.choose(name, value);
        self.settle();
    }

    pub fn event(&mut self, name: &str, data: Value) {
        self.tab.event(name, data);
        self.settle();
    }

    pub fn shows(&self, needle: &str) -> bool {
        self.tab.shows(needle)
    }

    /// Every text of the page, one per line, for a failure to print.
    pub fn page(&self) -> String {
        self.tab.texts().join("\n")
    }

    pub fn todo(&self) -> &Todo {
        self.tab.kit()
    }

    /// The `data.write` of `last.json`, last first.
    pub fn stored(&self) -> Option<Value> {
        self.files
            .get("last.json")
            .map(|text| serde_json::from_str(text).unwrap())
    }

    /// The last `tab.setBadge` params.
    pub fn badge(&self) -> Value {
        self.calls_to("tab.setBadge").pop().unwrap_or(Value::Null)
    }
}

/// The handlers of the buttons labelled `label`, in the order they are drawn;
/// none for a disabled one.
pub fn buttons<K: Kit>(tab: &Harness<K>, label: &str) -> Vec<Option<HandlerId>> {
    let mut found = Vec::new();
    tab.view().root.for_each(&mut |element| {
        if let Element::Button(button) = element
            && button.label.as_deref() == Some(label)
        {
            found.push(button.on_press);
        }
    });
    found
}

/// The tooltips of the buttons labelled `label`.
pub fn tooltips<K: Kit>(tab: &Harness<K>, label: &str) -> Vec<String> {
    let mut found = Vec::new();
    tab.view().root.for_each(&mut |element| {
        if let Element::Button(button) = element
            && button.label.as_deref() == Some(label)
        {
            found.extend(button.tooltip.clone());
        }
    });
    found
}

/// The filters drawn: label, count, pressed.
pub fn filters<K: Kit>(tab: &Harness<K>) -> Vec<(String, String, bool)> {
    let mut found = Vec::new();
    tab.view().root.for_each(&mut |element| {
        if let Element::Filter(filter) = element {
            found.push((
                filter.label.clone(),
                filter.count.clone().unwrap_or_default(),
                filter.pressed,
            ));
        }
    });
    found
}

/// The rows drawn for markers: those that have the hand-over button or its
/// answer.
pub fn rows<K: Kit>(tab: &Harness<K>) -> usize {
    let mut rows = 0;
    tab.view().root.for_each(&mut |element| {
        if let Element::Row(row) = element
            && row.children.iter().any(|child| {
                matches!(child, Element::Button(button)
                    if matches!(button.label.as_deref(), Some("Hand to an agent" | "Sending…")))
                    || matches!(child, Element::State(state) if state.label.starts_with("Sent to"))
            })
        {
            rows += 1;
        }
    });
    rows
}

/// A page element's text by prefix: the first text of the page that starts
/// with `prefix`.
pub fn text_starting<K: Kit>(tab: &Harness<K>, prefix: &str) -> Option<String> {
    tab.texts()
        .into_iter()
        .find(|text| text.starts_with(prefix))
}

/// Uses a value-less handler, for tests that hold a handler of an old view.
pub fn press_handler<K: Kit>(tab: &mut Harness<K>, handler: HandlerId) {
    tab.ui(handler, None::<EventValue>);
}
