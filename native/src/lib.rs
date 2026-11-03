//! The TODO tab, native: the `TODO`, `FIXME`, `HACK` and `XXX` comments of the
//! workspace, grouped by file or by tag, handed to an agent in one click.
//!
//! It is Rust built to WebAssembly (`scripts/build-native.sh` puts it at
//! `tabs/todo.wasm`, which the manifest's `entry` names), and Hangar 2.0 draws
//! it with its own widgets.
//!
//! - **State**: [`Todo`], changed by [`Message`]s in `update`, drawn by `view`.
//! - **Host calls**: `tab.setTitle`, `tab.setBadge`,
//!   `data.read` and `data.write` of `last.json`, `panes.list`,
//!   `command.run` of the declared `scan` (everything, then one file for its
//!   authors), `preview.show`, `panes.send`.
//! - **Subscriptions**: `panes.changed`, the language, whether the tab is in
//!   front, and a 30 s clock that keeps "Scanned 5 min ago" true.
//!
//! Every refusal is a state of the page, drawn: a `denied` says which
//! permission to accept, a failed scan says why and offers to try again.

pub mod logic;
pub mod strings;

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use hangar_kit_sdk::Handler;
use hangar_kit_sdk::api::{CommandAnswer, DataText, Done, Pane, Queued, Shown, Theme};
use hangar_kit_sdk::prelude::*;
use hangar_kit_sdk::ui::{self, ListItem};

use logic::{Authored, BlameStatus, GroupBy, Item, Scan, Tag};

/// The hash of the sources this module was built from. `scripts/build-native.sh`
/// sets it, and `scripts/check-native.sh` reads it back out of the committed
/// module to tell that the module is the sources'. It is also logged once, at
/// the start, so a person can tell which build runs.
const SOURCE: &str = match option_env!("KIT_TODO_SOURCE") {
    Some(hash) => hash,
    None => "unset",
};

/// How often the time in "Scanned 5 min ago" is brought up to date.
const TICK: Duration = Duration::from_secs(30);

/// What the tab is, before anything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The last scan and the agents are being read.
    Loading,
    Ready,
    /// `command:scan` is not accepted: the page says where to accept it.
    Denied,
}

/// A scan that did not complete: one of `timeout`, `not_found`, `busy`,
/// `unreadable`, `failed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunError {
    pub code: &'static str,
    pub detail: String,
}

/// What went wrong with the buttons of one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// `open`, `open_denied`, `denied`, `not_found`, `rate_limited`,
    /// `unavailable` or `failed`.
    pub code: &'static str,
    pub detail: String,
    pub agent: Option<String>,
}

/// Who wrote the markers of one file, asked for with its button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlameState {
    Loading,
    Denied,
    Done {
        status: BlameStatus,
        by_line: BTreeMap<i64, Authored>,
    },
}

pub struct Todo {
    pub t: Strings,
    pub utc_offset: i32,
    pub now: i64,
    pub visible: bool,
    pub screen: Screen,
    pub scanning: bool,
    /// The last scan, checked.
    pub scan: Option<Scan>,
    pub error: Option<RunError>,
    pub panes: Vec<Pane>,
    pub panes_denied: bool,
    /// The agent the person picked.
    pub choice: Option<String>,
    pub query: String,
    /// The tags kept; none means all.
    pub tags: Vec<Tag>,
    pub by: GroupBy,
    /// Rows drawn, grown by "Show more".
    pub limit: usize,
    /// Scans accepted so far: an answer about a marker of an older one is let go.
    pub scans: u32,
    /// Markers being sent, by key.
    pub pending: BTreeSet<usize>,
    /// Markers sent, by key, and to which agent.
    pub sent: BTreeMap<usize, String>,
    pub problems: BTreeMap<usize, Problem>,
    pub blame: BTreeMap<String, BlameState>,
    /// How many markers of the scan the stored copy holds, when not all.
    pub kept_partly: Option<usize>,
    /// Why the scan could not be stored at all.
    pub not_saved: Option<String>,
    last_read: bool,
    panes_read: bool,
    last_badge: Option<String>,
}

#[derive(Clone)]
pub enum Message {
    /// `last.json` read.
    LastRead(Result<DataText, KitError>),
    /// The agents listed.
    Panes(Result<Vec<Pane>, KitError>),
    PanesChanged,
    /// The scan button.
    Scan,
    Scanned(Result<CommandAnswer, KitError>),
    Stored(Result<Done, KitError>),
    Choose(String),
    Query(String),
    ToggleTag(Tag),
    ClearFilters,
    GroupBy(String),
    ShowMore,
    Open(usize),
    /// A file opened, of the scan numbered so.
    Opened(u32, usize, Result<Shown, KitError>),
    Blame(String),
    Blamed(String, Result<CommandAnswer, KitError>),
    HandOver(usize),
    /// A marker of the scan numbered so, handed to an agent.
    HandedOver(u32, usize, String, Result<Queued, KitError>),
    Tick,
    Visible(bool),
    Theme(Theme),
}

/// A detail of a failure, short enough to put in a sentence.
fn short(text: &str) -> String {
    text.chars().take(200).collect()
}

impl Todo {
    /// The agent a hand-over goes to: the one picked while it is still there,
    /// else the orchestrator, else the first reachable.
    pub fn target(&self) -> Option<String> {
        let reachable = logic::candidates(&self.panes);
        self.choice
            .clone()
            .filter(|choice| reachable.iter().any(|pane| &pane.name == choice))
            .or_else(|| logic::default_target(&self.panes))
    }

    /// The tab's badge now: running while it scans, else the scan's.
    fn set_badge(&mut self, cx: &mut Context<Message>) {
        let badge = if self.scanning {
            Some(logic::Badge {
                state: api::BadgeState::Running,
                text: None,
            })
        } else {
            logic::badge_for(self.scan.as_ref())
        };
        let id = match &badge {
            Some(badge) => format!("{:?}|{}", badge.state, badge.text.as_deref().unwrap_or("")),
            None => "none".to_owned(),
        };
        if self.last_badge.as_ref() == Some(&id) {
            return;
        }
        self.last_badge = Some(id);
        cx.call_quietly(api::TabSetBadge {
            state: badge.as_ref().map(|badge| badge.state),
            text: badge.and_then(|badge| badge.text),
        });
    }

    /// Both reads are in: the tab is ready, and scans, so that the list is
    /// current when the tab opens (reading files of the workspace is quick).
    fn ready(&mut self, cx: &mut Context<Message>) {
        if self.screen == Screen::Loading && self.last_read && self.panes_read {
            self.screen = Screen::Ready;
            self.set_badge(cx);
            self.run(cx);
        }
    }

    fn fail(&mut self, code: &'static str, detail: &str) {
        self.error = Some(RunError {
            code,
            detail: short(detail),
        });
    }

    /// Scans the workspace's files.
    fn run(&mut self, cx: &mut Context<Message>) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.error = None;
        if self.screen == Screen::Denied {
            self.screen = Screen::Ready;
        }
        self.set_badge(cx);
        cx.call(
            api::CommandRun {
                name: "scan".into(),
                args: BTreeMap::from([("file".to_owned(), String::new())]),
            },
            Message::Scanned,
        );
    }

    /// A scan that came out well: shown now, kept for the next opening.
    fn accept(&mut self, scan: Scan, cx: &mut Context<Message>) {
        self.scans += 1;
        self.pending.clear();
        self.sent.clear();
        self.problems.clear();
        self.blame.clear();
        self.limit = logic::SHOWN_STEP;
        // What the page shows stays whole; what is stored may be shorter, to
        // fit what `data.write` takes, and the page says so.
        let (text, kept) = scan.stored(logic::DATA_MAX_BYTES);
        self.kept_partly = (kept < scan.items.len()).then_some(kept);
        self.not_saved = None;
        self.scan = Some(scan);
        cx.call(
            api::DataWrite {
                path: logic::LAST.into(),
                text,
            },
            Message::Stored,
        );
    }

    fn scanned(&mut self, answer: Result<CommandAnswer, KitError>, cx: &mut Context<Message>) {
        match answer {
            Ok(answer) if answer.timed_out => self.fail("timeout", ""),
            Ok(answer) => {
                let ended = answer.exit_code == Some(0);
                match ended.then(|| logic::parse_scan(&answer.stdout)).flatten() {
                    Some(scan) => self.accept(scan, cx),
                    None => {
                        let detail = answer
                            .stderr
                            .lines()
                            .find(|line| !line.is_empty())
                            .map(str::to_owned)
                            .unwrap_or_else(|| {
                                format!(
                                    "exit code {}",
                                    answer
                                        .exit_code
                                        .map_or("none".to_owned(), |c| c.to_string())
                                )
                            });
                        self.fail(if ended { "unreadable" } else { "failed" }, &detail);
                    }
                }
            }
            Err(error) if error.is(KitError::DENIED) => self.screen = Screen::Denied,
            Err(error) if error.is(KitError::NOT_FOUND) => self.fail("not_found", ""),
            Err(error) if error.is(KitError::BUSY) => self.fail("busy", ""),
            Err(error) => self.fail("failed", &error.message),
        }
        self.scanning = false;
        self.now = cx.now();
        self.set_badge(cx);
    }

    fn blamed(&mut self, file: String, answer: Result<CommandAnswer, KitError>) {
        let failed = BlameState::Done {
            status: BlameStatus::Error,
            by_line: BTreeMap::new(),
        };
        let state = match answer {
            Ok(answer) if answer.exit_code == Some(0) && !answer.timed_out => {
                match logic::parse_blame(&answer.stdout) {
                    Some(blame) => BlameState::Done {
                        status: blame.status,
                        by_line: blame.by_line,
                    },
                    None => failed,
                }
            }
            Err(error) if error.is(KitError::DENIED) => BlameState::Denied,
            _ => failed,
        };
        self.blame.insert(file, state);
    }
}

impl Kit for Todo {
    type Message = Message;

    fn init(start: &Start, cx: &mut Context<Message>) -> Self {
        console::debug(format!("kit-todo source {SOURCE}"));
        let t = strings::strings(&start.theme.locale);
        cx.call_quietly(api::TabSetTitle { title: t.t("tab") });
        cx.call(
            api::DataRead {
                path: logic::LAST.into(),
            },
            Message::LastRead,
        );
        cx.call(api::PanesList {}, Message::Panes);
        Todo {
            t,
            utc_offset: start.utc_offset,
            now: cx.now(),
            visible: start.visible,
            screen: Screen::Loading,
            scanning: false,
            scan: None,
            error: None,
            panes: Vec::new(),
            panes_denied: false,
            choice: None,
            query: String::new(),
            tags: Vec::new(),
            by: GroupBy::File,
            limit: logic::SHOWN_STEP,
            scans: 0,
            pending: BTreeSet::new(),
            sent: BTreeMap::new(),
            problems: BTreeMap::new(),
            blame: BTreeMap::new(),
            kept_partly: None,
            not_saved: None,
            last_read: false,
            panes_read: false,
            last_badge: None,
        }
    }

    fn update(&mut self, message: Message, cx: &mut Context<Message>) {
        match message {
            // Nothing scanned yet, or a data folder that is not ours to read:
            // the page works all the same.
            Message::LastRead(answer) => {
                self.scan = answer.ok().and_then(|read| logic::parse_scan(&read.text));
                self.last_read = true;
                self.ready(cx);
            }
            Message::Panes(answer) => {
                match answer {
                    Ok(panes) => {
                        self.panes = panes;
                        self.panes_denied = false;
                    }
                    Err(error) => {
                        self.panes = Vec::new();
                        self.panes_denied = error.is(KitError::DENIED);
                    }
                }
                self.panes_read = true;
                self.ready(cx);
            }
            Message::PanesChanged => cx.call(api::PanesList {}, Message::Panes),
            Message::Scan => self.run(cx),
            Message::Scanned(answer) => self.scanned(answer, cx),
            Message::Stored(Ok(_)) => {}
            // It shows now; the next opening starts without it. The page says so.
            Message::Stored(Err(error)) => self.not_saved = Some(short(&error.message)),
            Message::Choose(agent) => self.choice = Some(agent),
            Message::Query(query) => {
                self.query = query.chars().take(200).collect();
                self.limit = logic::SHOWN_STEP;
            }
            Message::ToggleTag(tag) => {
                if self.tags.contains(&tag) {
                    self.tags.retain(|kept| *kept != tag);
                } else {
                    self.tags.push(tag);
                }
                self.limit = logic::SHOWN_STEP;
            }
            Message::ClearFilters => {
                self.query.clear();
                self.tags.clear();
                self.limit = logic::SHOWN_STEP;
            }
            Message::GroupBy(by) => {
                self.by = GroupBy::parse(&by);
                self.limit = logic::SHOWN_STEP;
            }
            Message::ShowMore => self.limit += logic::SHOWN_STEP,
            Message::Open(key) => {
                let Some(marker) = self.scan.as_ref().and_then(|scan| scan.find(key)) else {
                    return;
                };
                if !logic::can_open(&marker.file) {
                    return;
                }
                let path = marker.file.clone();
                self.problems.remove(&key);
                let scans = self.scans;
                cx.call(api::PreviewShow { path }, move |answer| {
                    Message::Opened(scans, key, answer)
                });
            }
            // The answer to a marker of a scan that is gone says nothing of this one.
            Message::Opened(scans, ..) | Message::HandedOver(scans, ..) if scans != self.scans => {}
            Message::Opened(_, key, Err(error)) => {
                let code = if error.is(KitError::DENIED) {
                    "open_denied"
                } else {
                    "open"
                };
                self.problems.insert(
                    key,
                    Problem {
                        code,
                        detail: short(&error.message),
                        agent: None,
                    },
                );
            }
            Message::Opened(_, _, Ok(_)) => {}
            // Who wrote the markers of one file, from git, once.
            Message::Blame(file) => {
                if self.scan.is_none() || self.blame.get(&file) == Some(&BlameState::Loading) {
                    return;
                }
                self.blame.insert(file.clone(), BlameState::Loading);
                // A value that starts with a dash is given as a path, never as an option.
                let asked = if file.starts_with('-') {
                    format!("./{file}")
                } else {
                    file.clone()
                };
                cx.call(
                    api::CommandRun {
                        name: "scan".into(),
                        args: BTreeMap::from([("file".to_owned(), asked)]),
                    },
                    move |answer| Message::Blamed(file, answer),
                );
            }
            Message::Blamed(file, answer) => self.blamed(file, answer),
            Message::HandOver(key) => {
                let agent = self.target();
                let Some(marker) = self.scan.as_ref().and_then(|scan| scan.find(key)) else {
                    return;
                };
                let Some(agent) = agent else { return };
                if self.pending.contains(&key) {
                    return;
                }
                let text = logic::brief_for(marker);
                self.pending.insert(key);
                self.problems.remove(&key);
                let pane = agent.clone();
                let scans = self.scans;
                cx.call(api::PanesSend { pane, text }, move |answer| {
                    Message::HandedOver(scans, key, agent, answer)
                });
            }
            Message::HandedOver(_, key, agent, answer) => {
                self.pending.remove(&key);
                match answer {
                    Ok(_) => {
                        self.sent.insert(key, agent);
                    }
                    Err(error) => {
                        let code = ["denied", "not_found", "rate_limited", "unavailable"]
                            .into_iter()
                            .find(|code| error.is(code))
                            .unwrap_or("failed");
                        self.problems.insert(
                            key,
                            Problem {
                                code,
                                detail: short(&error.message),
                                agent: Some(agent),
                            },
                        );
                        // The agent went away: the list of agents is stale.
                        if code == "not_found" || code == "unavailable" {
                            cx.call(api::PanesList {}, Message::Panes);
                        }
                    }
                }
            }
            Message::Tick => self.now = cx.now(),
            Message::Visible(visible) => {
                self.visible = visible;
                self.now = cx.now();
            }
            // Hangar's language changed: the strip's title and badge follow.
            Message::Theme(theme) => {
                let language = |locale: &str| locale.split(['-', '_']).next().map(str::to_owned);
                let before = language(self.t.locale());
                self.t.set_locale(theme.locale);
                if language(self.t.locale()) != before {
                    cx.call_quietly(api::TabSetTitle {
                        title: self.t.t("tab"),
                    });
                    self.last_badge = None;
                    self.set_badge(cx);
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let t = &self.t;
        let mut parts: Vec<Element<Message>> = vec![self.header().into()];
        match self.screen {
            Screen::Loading => parts.push(loading(t.t("scanning")).into()),
            Screen::Denied => parts.push(
                denied(t.t("deniedTitle"))
                    .text(t.t("deniedText"))
                    .action(button(t.t("retry")).on_press(Message::Scan))
                    .into(),
            ),
            Screen::Ready => parts.extend(self.body()),
        }
        page(parts).into()
    }

    fn subscriptions(&self) -> Subscriptions<Message> {
        let mut subscriptions = Subscriptions::none()
            .visibility(Message::Visible)
            .theme(Message::Theme);
        // The agents are read again whenever they change, once they were read.
        if self.screen != Screen::Loading {
            subscriptions = subscriptions.panes_changed(Message::PanesChanged);
        }
        // The clock only keeps the ages true: behind another tab, nobody reads them.
        if self.visible {
            subscriptions = subscriptions.every("clock", TICK, Message::Tick);
        }
        subscriptions
    }
}

// ---------------------------------------------------------------- The page.

impl Todo {
    fn since(&self, at: i64) -> String {
        format::since(at, self.now, self.utc_offset, self.t.locale())
    }

    fn header(&self) -> ui::Header<Handler<Message>> {
        let t = &self.t;
        let mut header = header(t.t("tab"));
        if self.screen != Screen::Loading {
            if self.scanning {
                header = header.state(state_label(Status::Running, t.t("scanning")));
            } else if let Some(scan) = &self.scan {
                header = header.state(if scan.total > 0 {
                    state_label(Status::Neutral, t.tn("stateFound", scan.total as i64, &[]))
                } else {
                    state_label(Status::Success, t.t("stateNone"))
                });
                let when = self.since(scan.at);
                header = header.state(text(t.tv("lastScan", &[("time", &when)])).small().dim());
            }
        }
        if self.screen == Screen::Ready {
            let label = if self.scan.is_some() {
                t.t("scanAgain")
            } else {
                t.t("scan")
            };
            header = header.action(
                button(label)
                    .small()
                    .on_press_maybe((!self.scanning).then_some(Message::Scan)),
            );
        }
        header
    }

    /// Everything under the header of a tab that is ready.
    fn body(&self) -> Vec<Element<Message>> {
        let t = &self.t;
        let mut parts: Vec<Element<Message>> = Vec::new();
        if self.scanning {
            parts.push(progress(None).label(t.t("scanning")).into());
            if self.scan.is_none() {
                parts.push(banner(Notice::Info, t.t("scanningText")).into());
            }
        }
        if let Some(error) = &self.error {
            let mut state = failed(t.t("errorTitle")).text(t.tv(
                &format!("error.{}", error.code),
                &[("detail", &error.detail)],
            ));
            if !self.scanning {
                state = state.action(
                    button(t.t("retry"))
                        .primary()
                        .small()
                        .on_press(Message::Scan),
                );
            }
            parts.push(state.into());
        }
        let Some(scan) = &self.scan else {
            return parts;
        };
        if let Some(kept) = self.kept_partly {
            parts.push(
                banner(
                    Notice::Warning,
                    t.tv(
                        "savedPartly",
                        &[
                            ("kept", &kept.to_string()),
                            ("shown", &scan.items.len().to_string()),
                        ],
                    ),
                )
                .into(),
            );
        }
        if let Some(why) = &self.not_saved {
            parts.push(banner(Notice::Warning, t.tv("notSaved", &[("detail", why)])).into());
        }
        if scan.total == 0 {
            if scan.incomplete {
                parts.push(banner(Notice::Warning, t.t("incomplete")).into());
            }
            parts.push(
                empty(t.t("noneTitle"))
                    .icon(Icon::Check)
                    .text(t.t("noneText"))
                    .into(),
            );
            return parts;
        }
        parts.push(self.results(scan));
        parts
    }

    /// The markers: warnings, the controls, and the rows.
    fn results(&self, scan: &Scan) -> Element<Message> {
        let t = &self.t;
        let mut parts: Vec<Element<Message>> = Vec::new();
        if scan.truncated || scan.total > scan.items.len() {
            let total = scan.total.to_string();
            let shown = scan.items.len().to_string();
            parts.push(
                banner(
                    Notice::Warning,
                    t.tv("tooMany", &[("total", &total), ("shown", &shown)]),
                )
                .title(format!("{}.", t.t("tooManyTitle")))
                .into(),
            );
        }
        if scan.incomplete {
            parts.push(banner(Notice::Warning, t.t("incomplete")).into());
        }
        parts.push(self.controls(scan));
        parts.push(self.agent_picker());
        let matching = logic::filter_items(&scan.items, &self.query, &self.tags);
        if matching.is_empty() {
            parts.push(
                empty(t.t("nothingMatches"))
                    .icon(Icon::Search)
                    .text(t.t("nothingMatchesText"))
                    .action(button(t.t("clearFilters")).on_press(Message::ClearFilters))
                    .into(),
            );
        } else {
            parts.push(
                text(t.tn(
                    "matches",
                    matching.len() as i64,
                    &[("total", &scan.items.len().to_string())],
                ))
                .small()
                .dim()
                .into(),
            );
            parts.extend(self.groups(&matching));
        }
        column(parts).spacing(12.0).into()
    }

    /// One filter per tag found (with the scan's complete count), the search,
    /// and how to group.
    fn controls(&self, scan: &Scan) -> Element<Message> {
        let t = &self.t;
        let tags = Tag::ALL.into_iter().filter(|tag| scan.count(*tag) > 0).map(
            |tag| -> Element<Message> {
                filter(tag.as_str(), self.tags.contains(&tag))
                    .count(scan.count(tag).to_string())
                    .on_press(Message::ToggleTag(tag))
                    .into()
            },
        );
        col![
            row(tags).spacing(6.0).wrap(),
            search(t.t("search"), self.query.as_str())
                .id("q")
                .on_input(Message::Query)
                .on_clear(Message::Query(String::new())),
            select(
                [
                    select_option(GroupBy::File.as_str(), t.t("byFile")),
                    select_option(GroupBy::Tag.as_str(), t.t("byTag")),
                ],
                Some(self.by.as_str()),
            )
            .label(t.t("groupLabel"))
            .on_select(Message::GroupBy),
        ]
        .spacing(8.0)
        .into()
    }

    /// One line saying who the markers go to: a choice when there are several
    /// agents.
    fn agent_picker(&self) -> Element<Message> {
        let t = &self.t;
        if self.panes_denied {
            return text(t.t("panesDenied")).small().dim().into();
        }
        let reachable = logic::candidates(&self.panes);
        match reachable.as_slice() {
            [] => text(t.t("noAgent")).small().dim().into(),
            [only] => text(format!("{} : {}", t.t("agentLabel"), only.name))
                .small()
                .dim()
                .into(),
            several => select(
                several
                    .iter()
                    .map(|pane| select_option(pane.name.clone(), pane.name.clone())),
                self.target(),
            )
            .label(t.t("agentLabel"))
            .on_select(Message::Choose)
            .into(),
        }
    }

    /// The groups' rows, as many as the limit allows, and a button for the rest.
    fn groups(&self, matching: &[&Item]) -> Vec<Element<Message>> {
        let t = &self.t;
        let target = self.target();
        let mut left = self.limit;
        let mut parts: Vec<Element<Message>> = Vec::new();
        for group in logic::group(matching, self.by) {
            if left == 0 {
                break;
            }
            let shown = &group.items[..group.items.len().min(left)];
            left -= shown.len();
            let rows = list(
                shown
                    .iter()
                    .map(|marker| self.row(marker, target.is_some())),
            )
            .group();
            let count = group.items.len().to_string();
            match self.by {
                GroupBy::Tag => {
                    parts.push(section(group.id.clone(), [rows.into()]).count(count).into())
                }
                GroupBy::File => {
                    let mut block: Vec<Element<Message>> = vec![self.file_head(&group.id, count)];
                    match self.blame.get(&group.id) {
                        Some(BlameState::Denied) => {
                            block.push(banner(Notice::Error, t.t("blameDenied")).into())
                        }
                        Some(BlameState::Done { status, .. }) if *status != BlameStatus::Ok => {
                            let key = match status {
                                BlameStatus::Untracked => "blameStatus_untracked",
                                BlameStatus::NoGit => "blameStatus_no_git",
                                _ => "blameStatus_error",
                            };
                            block.push(banner(Notice::Warning, t.t(key)).into());
                        }
                        _ => {}
                    }
                    block.push(rows.into());
                    parts.push(column(block).spacing(8.0).into());
                }
            }
        }
        let shown = self.limit.min(matching.len());
        if shown < matching.len() {
            parts.push(
                button(t.tv("showMore", &[("n", &(matching.len() - shown).to_string())]))
                    .small()
                    .ghost()
                    .on_press(Message::ShowMore)
                    .into(),
            );
        }
        parts
    }

    /// The title of a file's group: its path in the terminals' face, how many
    /// markers, and the button that asks git who wrote them.
    fn file_head(&self, file: &str, count: String) -> Element<Message> {
        let t = &self.t;
        let asked = self.blame.get(file);
        let mut head = item(file).mono().wrap().end(text(count).small().muted());
        if !matches!(asked, Some(BlameState::Done { .. })) {
            let loading = asked == Some(&BlameState::Loading);
            head = head.action(
                button(if loading {
                    t.t("authorLoading")
                } else {
                    t.t("author")
                })
                .small()
                .ghost()
                .tooltip(t.tv("authorLabel", &[("file", file)]))
                .on_press_maybe((!loading).then(|| Message::Blame(file.to_owned()))),
            );
        }
        list([head]).into()
    }

    /// Who wrote a marker and when, once its file was blamed.
    fn author_line(&self, marker: &Item) -> Option<Element<Message>> {
        let t = &self.t;
        let Some(BlameState::Done { by_line, .. }) = self.blame.get(&marker.file) else {
            return None;
        };
        let entry = by_line.get(&marker.line)?;
        let line = match &entry.author {
            None => t.t("authorNotCommitted"),
            Some(author) => {
                let when = entry.at.map(|at| self.since(at)).unwrap_or_default();
                let line = t.tv("authorBy", &[("author", author), ("time", &when)]);
                line.strip_suffix(", ").map(str::to_owned).unwrap_or(line)
            }
        };
        Some(text(line).small().dim().into())
    }

    /// One marker: its comment, its tag and place, and its two buttons.
    fn row(&self, marker: &Item, can_send: bool) -> ListItem<Handler<Message>> {
        let t = &self.t;
        let key = marker.key;
        let title = if marker.text.is_empty() {
            t.t("noText")
        } else {
            marker.text.clone()
        };
        let place = if self.by == GroupBy::File {
            format!(":{}", marker.line)
        } else {
            format!("{}:{}", marker.file, marker.line)
        };
        let mut row_item = item(title).wrap().body(
            row![chip(marker.tag.as_str()), text(place).small().dim().mono()]
                .spacing(6.0)
                .align(Align::Center)
                .wrap(),
        );
        if let Some(author) = self.author_line(marker) {
            row_item = row_item.body(author);
        }
        if let Some(problem) = self.problems.get(&key) {
            let agent = problem.agent.as_deref().unwrap_or("");
            let message = t.tv(
                &format!("problem.{}", problem.code),
                &[("agent", agent), ("detail", &problem.detail)],
            );
            row_item = row_item.body(banner(Notice::Error, message));
        }
        let mut buttons: Vec<Element<Message>> = Vec::new();
        if logic::can_open(&marker.file) {
            buttons.push(
                button(t.t("open"))
                    .small()
                    .ghost()
                    .tooltip(t.tv(
                        "openLabel",
                        &[("file", &format!("{}:{}", marker.file, marker.line))],
                    ))
                    .on_press(Message::Open(key))
                    .into(),
            );
        }
        buttons.push(match self.sent.get(&key) {
            Some(agent) => state_label(Status::Success, t.tv("sentTo", &[("agent", agent)])).into(),
            None => {
                let pending = self.pending.contains(&key);
                button(if pending {
                    t.t("handingOver")
                } else {
                    t.t("handOver")
                })
                .small()
                .on_press_maybe((can_send && !pending).then_some(Message::HandOver(key)))
                .into()
            }
        });
        row_item.body(row(buttons).spacing(6.0).align(Align::Center).wrap())
    }
}

export_kit!(Todo);
