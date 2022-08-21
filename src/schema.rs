use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetShortcut {
    pub slug: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PutShortcut {
    pub url: String,
}
#[derive(Debug, Serialize)]
pub struct PutShortcutAnwser {
    pub slug: String,
}
