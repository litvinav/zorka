use crate::{configuration::get_config, routes::*};
use actix_web::{middleware::Logger, web, App, HttpServer};
use tera::Tera;

mod configuration;
mod database;
mod routes;
mod schema;
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("zorka=error"));

    let db_filename = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL env is required to target the sqlite database");
    let pool = database::setup_database(db_filename).await;
    let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let config = get_config();

    log::info!("Starting HTTP server at http://127.1:8080");
    HttpServer::new(move || {
        App::new()
            .service(health)
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            .service(find)
            .service(create)
            .service(delete)
            .service(assets)
            .service(render)
            .service(dashboard)
            .service(code)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))
    .expect("Could not bind the http server on port 8080")
    .run()
    .await
}
