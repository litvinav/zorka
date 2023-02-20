use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetShortcut {
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct PutShortcut {
    pub slug: String,
    pub url: String,
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
