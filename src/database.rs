use regex::Regex;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::{
    fs,
    io::{BufRead, BufReader},
};

async fn migrate(pool: &Pool<Sqlite>) {
    let qry = "CREATE TABLE IF NOT EXISTS shortcut (
        slug    TEXT    PRIMARY KEY     NOT NULL,
        url     TEXT                    NOT NULL,
        status  TEXT                    NOT NULL,
        since   TEXT                    NOT NULL,
        until   TEXT                    NOT NULL
    );
    CREATE UNIQUE INDEX IF NOT EXISTS indx_slug ON shortcut (slug);";
    if let Err(e) = sqlx::query(qry).execute(pool).await {
        log::error!("{}", e.to_string());
    }
}

async fn seed(pool: &Pool<Sqlite>) {
    match fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/seed.csv")) {
        Ok(file) => {
            let buf = BufReader::new(file);
            let regex =
                Regex::new(
                    r"^(?P<slug>[a-z0-9]+),(?P<url>https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()!@:%_\+.~#?&//=]*)),(?P<status>((un)?trusted)),(?P<since>\d+),(?P<until>\d+)$"
                ).expect("invalid regex");

            let mut data: Vec<String> = vec![];
            for content in buf.lines().flatten() {
                if let Some(capture) = regex.captures(&content) {
                    if let (Some(slug), Some(url), Some(status), Some(since), Some(until)) = (
                        capture.name("slug"),
                        capture.name("url"),
                        capture.name("status"),
                        capture.name("since"),
                        capture.name("until"),
                    ) {
                        data.push(format!(
                            "('{}','{}','{}','{}','{}')",
                            slug.as_str(),
                            url.as_str(),
                            status.as_str(),
                            since.as_str(),
                            until.as_str()
                        ));
                    }
                }
            }
            let qry = format!("INSERT OR REPLACE INTO shortcut VALUES {};", data.join(","));
            match sqlx::query(&qry).execute(pool).await {
                Ok(result) => log::debug!("Imported {} items.", result.rows_affected()),
                Err(e) => log::error!("{}", e.to_string()),
            }
        }
        Err(_) => log::debug!("Skipping seeding since no seed.csv was found."),
    }
}

pub async fn setup_database(db_filename: String) -> Pool<Sqlite> {
    if !Sqlite::database_exists(&db_filename).await.unwrap_or(false) {
        Sqlite::create_database(&db_filename)
            .await
            .expect("New database could not be created from the provided connection string.");
    }
    let pool = SqlitePoolOptions::new()
        .connect(&db_filename)
        .await
        .expect("Could not connect to the database.");
    migrate(&pool).await;
    seed(&pool).await;

    pool
}
