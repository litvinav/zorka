use crate::{configuration::get_config, routes::*};
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_files::Files;
use tera::Tera;

mod configuration;
mod database;
mod routes;
mod schema;
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "zorka=error".to_string());
    env_logger::init_from_env(log_level);

    log::info!("Starting HTTP server at http://127.1:8080");
    HttpServer::new(move || {
        App::new()
            .service(health)
            .app_data(web::Data::new(get_config()))
            .app_data(web::Data::new(database::setup()))
            .app_data(web::Data::new(Tera::new("./templates/**/*").unwrap()))
            .service(find)
            .service(create)
            .service(delete)
            .service(share)
            .service(render)
            .service(dashboard)
            .service(code)
            .service(Files::new("/assets/", "./assets/").disable_content_disposition())
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))
    .expect("Could not bind the http server on port 8080")
    .run()
    .await
}
