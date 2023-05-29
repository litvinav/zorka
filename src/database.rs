use regex::Regex;
use std::fs::OpenOptions;
use std::io::Write;
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
    instance_id: String,
    data: Arc<RwLock<HashMap<String, ShortcutEntry>>>,
}

impl Database {
    pub fn new(data: HashMap<String, ShortcutEntry>, instance_id: String) -> Self {
        Self {
            instance_id,
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

    pub fn backup(&self) -> Result<(), std::io::Error> {
        match std::fs::metadata("./backups") {
            Ok(meta) if !meta.is_dir() => std::fs::create_dir("backups").unwrap(),
            Err(_) => std::fs::create_dir("backups").unwrap(),
            _ => {}
        };

        std::fs::write(format!("./backups/{}.csv", self.instance_id), self.to_csv())
    }
}

pub fn setup() -> Database {
    let instance_id = Uuid::new_v4().to_string();
    let data = restore_data(&instance_id);
    if let Ok(backup_period) = std::env::var("BACKUP_CRON") {
        println!(
            "echo '{backup_period} echo curl http://127.1:8080/backup' > /etc/cron.d/zorka_backup"
        );
    }
    Database::new(data, instance_id)
}

#[cfg(not(test))]
impl Drop for Database {
    fn drop(&mut self) {
        println!("Backing up database pre shutdown...");
        self.backup()
            .expect("could not backup the database on exit");
    }
}

fn restore_data(instance_id: &String) -> HashMap<String, ShortcutEntry> {
    let mut data: HashMap<String, ShortcutEntry> = HashMap::new();

    let mut relevant_targets: Vec<String> = vec![];
    if let Ok(structure) = std::fs::read_to_string("./backup.zorka") {
        for entry in structure.split('\n') {
            if entry.len() > 36 {
                let deprecators: Vec<&str> = entry[49..entry.len()].split(',').collect();
                for i in relevant_targets.len()..0 {
                    if deprecators.contains(&relevant_targets[i].as_str()) {
                        relevant_targets.remove(i);
                    }
                }
            }
            if entry.len() >= 36 {
                relevant_targets.push(entry[0..36].to_string());
            }
        }
    } else {
        std::fs::write("./backup.zorka", "")
            .expect("cannot creat backup.zorka in a read-only filesystem");
    }

    let mut files_to_process = vec![];
    for uuid in &relevant_targets {
        let path = format!("./backups/{uuid}.csv");
        match std::fs::metadata(&path) {
            Ok(meta) if meta.is_file() => files_to_process.push(path),
            _ => println!("could not load {path}"),
        }
    }

    let mut deprecated = "".into();
    if files_to_process.is_empty() {
        if let Ok(meta) = std::fs::metadata("./seed.csv") {
            if meta.is_file() {
                load_data(&mut data, vec!["./seed.csv".into()]);
            }
        }
    } else {
        load_data(&mut data, files_to_process);
        deprecated = format!(" DEPRECATED: {}", relevant_targets.join(","));
    }

    OpenOptions::new()
        .write(true)
        .append(true)
        .open("./backup.zorka")
        .expect("could not open the backup zorka file")
        .write_all(format!("{instance_id}{}\n", deprecated).as_bytes())
        .expect("could not create a new entry in backup.zorka");

    data
}

fn load_data(data: &mut HashMap<String, ShortcutEntry>, files: Vec<String>) {
    for path in files {
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
            Err(e) => println!("Skipping seeding {path}: {e}"),
        }
    }
}
