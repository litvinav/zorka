use regex::Regex;
use std::fs::{metadata, remove_file};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use std::{
    fs,
    io::{BufRead, BufReader},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct ShortcutEntry {
    pub slug: String,
    pub url: String,
    pub status: String,
    pub since: String,
    pub until: String,
}

pub struct Database {
    backup: bool,
    instance_id: String,
    data: Arc<RwLock<HashMap<String, ShortcutEntry>>>,
}

impl Database {
    pub fn new(backup: bool) -> Self {
        Self {
            backup,
            data: Arc::new(RwLock::new(restore_data())),
            instance_id: Uuid::new_v4().to_string(),
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

    pub fn to_csv(&self) -> String {
        self.read_all()
            .iter()
            .map(|row| {
                format!(
                    "{},{},{},{},{}",
                    row.slug, row.url, row.status, row.since, row.until,
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if self.backup {
            println!("Backing up database pre shutdown...");
            match std::fs::create_dir("backups") {
                Err(e) if e.kind() != ErrorKind::AlreadyExists => println!("{e}"),
                _ => {}
            };
            std::fs::write(format!("./backups/{}.csv", self.instance_id), self.to_csv())
                .expect("could not backup the database on exit");
        }
    }
}

fn restore_data() -> HashMap<String, ShortcutEntry> {
    let mut data: HashMap<String, ShortcutEntry> = HashMap::new();
    let files = match std::fs::read_dir("./backups") {
        Ok(entries) => {
            let mut paths = vec![];
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.metadata().unwrap().is_file() {
                        paths.push(entry.path());
                    }
                }
            }
            paths
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                match std::fs::create_dir("./backups") {
                    Err(e) if e.kind() != ErrorKind::AlreadyExists => println!("{e}"),
                    _ => {},
                }
            } else {
                println!("{e}");
            }
            vec![]
        }
    };

    if files.is_empty() {
        match metadata("./seed.csv") {
            Ok(meta) if meta.is_file() => load_data(&mut data, &PathBuf::from("./seed.csv")),
            _ => {},
        }
    } else {
        for path in files {
            load_data(&mut data, &path);
            if let Err(e) = remove_file(&path) {
                println!("{e}");
            }
        }
    }
    data
}

fn load_data(data: &mut HashMap<String, ShortcutEntry>, path: &PathBuf) {
    match fs::File::open(&path) {
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
        Err(e) => println!("Skipping seeding {path:?}: {e}"),
    }
}
