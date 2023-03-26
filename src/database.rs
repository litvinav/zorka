use crate::schema::ShortcutEntry;
use regex::Regex;
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use std::{
    fs,
    io::{BufRead, BufReader},
};

fn seeded() -> HashMap<String, ShortcutEntry> {
    let mut data: HashMap<String, ShortcutEntry> = HashMap::new();
    match fs::File::open("./seed.csv") {
        Ok(file) => {
            let buf = BufReader::new(file);
            let regex =
                Regex::new(
                    r"^(?P<slug>[a-z0-9]+),(?P<url>https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()!@:%_\+.~#?&//=]*)),(?P<status>((un)?trusted)),(?P<since>\d+),(?P<until>\d+)$"
                ).expect("invalid regex");
            for content in buf.lines().flatten() {
                if let Some(capture) = regex.captures(&content) {
                    if let (Some(slug), Some(url), Some(status), Some(since), Some(until)) = (
                        capture.name("slug"),
                        capture.name("url"),
                        capture.name("status"),
                        capture.name("since"),
                        capture.name("until"),
                    ) {
                        data.insert(
                            slug.as_str().to_string(),
                            ShortcutEntry {
                                slug: slug.as_str().to_string(),
                                url: url.as_str().to_string(),
                                status: status.as_str().to_string(),
                                since: since.as_str().to_string(),
                                until: until.as_str().to_string(),
                            },
                        );
                    }
                }
            }
        }
        Err(_) => log::debug!("Skipping seeding since no seed.csv was found."),
    }
    data
}

#[derive(Clone)]
pub struct Database {
    data: Arc<RwLock<HashMap<String, ShortcutEntry>>>,
}

impl Database {
    pub fn new(data: HashMap<String, ShortcutEntry>) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
        }
    }
    pub fn read(&self, slug: &String) -> Option<ShortcutEntry> {
        if let Ok(data) = self.data.read() {
            data.get(slug).map(|entry| ShortcutEntry {
                slug: entry.slug.clone(),
                url: entry.url.clone(),
                status: entry.status.clone(),
                since: entry.since.clone(),
                until: entry.until.clone(),
            })
        } else {
            None
        }
    }
    pub fn read_all(&self) -> Vec<ShortcutEntry> {
        let mut all: Vec<ShortcutEntry> = vec![];
        if let Ok(data) = self.data.read() {
            for (_, entry) in data.iter() {
                all.push(entry.clone())
            }
        }
        all
    }
    pub fn upsert(&self, slug: String, value: ShortcutEntry) -> bool {
        if let Ok(mut locked) = self.data.write() {
            locked.insert(slug, value);
            true
        } else {
            false
        }
    }
    pub fn delete(&self, slug: &String) -> bool {
        if let Ok(mut locked) = self.data.write() {
            locked.remove(slug).is_some()
        } else {
            false
        }
    }
}

pub fn setup() -> Database {
    let data = seeded();
    Database::new(data)
}
