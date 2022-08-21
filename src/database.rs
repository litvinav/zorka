use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};

async fn migrate(db_filename: &str) {
    let pool = sqlx::SqlitePool::connect(&db_filename).await.unwrap();
    let qry = "CREATE TABLE IF NOT EXISTS shortcut
    (
        slug        VARCHAR(32) PRIMARY KEY     NOT NULL,
        url         TEXT                        NOT NULL
    );";
    let result = sqlx::query(&qry).execute(&pool).await;
    log::debug!("{:?}", result);
    pool.close().await;
}

pub async fn setup_database(db_filename: String) -> Pool<Sqlite> {
    if !Sqlite::database_exists(&db_filename).await.unwrap_or(false) {
        Sqlite::create_database(&db_filename)
            .await
            .expect("New database could not be created from the provided connection string");
    }
    migrate(&db_filename).await;

    SqlitePoolOptions::new().connect(&db_filename).await.expect("Could not connect to the database")
}
