use crate::routes::{create, delete, find, health};
use actix_web::{middleware::Logger, web, App, HttpServer};

mod database;
mod routes;
mod schema;
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("zorka=error"));

    log::info!("Starting HTTP server at http://0:8080");

    let db_filename = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL env is required to target the sqlite database");
    let pools = database::setup_database(db_filename).await;

    HttpServer::new(move || {
        App::new()
            .service(health)
            .app_data(web::Data::new(pools.clone()))
            .service(find)
            .service(create)
            .service(delete)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))
    .expect("Could not bind the http server on port 8080")
    .run()
    .await
}
