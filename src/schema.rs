use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetShortcut {
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct PutShortcut {
    pub slug: String,
    pub url: String,
    pub approval: bool,
    pub since: u128,
    pub until: u128,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PutShortcutAnwser {
    pub slug: String,
}
#[derive(Debug, Deserialize)]
pub struct DeleteShortcut {
    pub slug: String,
}

// DASHBOARD CONTEXT

#[derive(Serialize)]
pub struct ShortcutItem {
    pub slug: String,
    pub url: String,
    pub status: String,
    pub now: u128,
    pub since: u128,
    pub until: u128,
}

#[derive(Serialize)]
pub struct ShortcutList {
    pub items: Vec<ShortcutItem>,
}

// GATE CONTEXT
#[derive(Serialize)]
pub struct Approval {
    pub url: String,
    pub lang: String,
    pub dir: String,
    pub label: String,
    pub button: String,
}

#[derive(Serialize)]
pub struct Blocker {
    pub lang: String,
    pub dir: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct Countdown {
    pub timestamp: u128,
    pub lang: String,
    pub dir: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct Share {
    pub slug: String,
    pub vector: String,
}

// DATA

#[derive(Clone)]
pub struct ShortcutEntry {
    pub slug: String,
    pub url: String,
    pub status: String,
    pub since: String,
    pub until: String,
}
