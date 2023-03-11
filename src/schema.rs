use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Assets {
    pub file: String,
}

#[derive(Deserialize)]
pub struct GetShortcut {
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct PutShortcut {
    pub slug: String,
    pub url: String,
    pub since: u128,
    pub until: u128,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PutShortcutAnwser {
    pub slug: String,
}
#[derive(Debug, Deserialize)]
pub struct DeleteShortcut {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteShortcutAnwser {
    pub rows_affected: u64,
}

// - - -

#[derive(Serialize)]
pub struct ShortcutItem {
    pub slug: String,
    pub url: String,
    pub now: u128,
    pub since: u128,
    pub until: u128,
}

#[derive(Serialize)]
pub struct ShortcutList {
    pub items: Vec<ShortcutItem>,
}

#[derive(Serialize)]
pub struct Countdown {
    pub timestamp: u128,
}
