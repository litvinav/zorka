use crate::{configuration::get_config, routes::*};
use actix_files::Files;
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::sync::Arc;
use tera::Tera;

mod configuration;
mod database;
mod routes;
mod schema;
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_config();
    let database = Arc::new(database::setup());

    println!("Starting HTTP server at http://0:8080");
    HttpServer::new(move || {
        App::new()
            .service(health)
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(
                Tera::new("./templates/**/*").expect("no templates found"),
            ))
            .service(find)
            .service(create)
            .service(delete)
            .service(share)
            .service(store)
            .service(dashboard)
            .service(code)
            .service(Files::new("/assets/", "./assets/").disable_content_disposition())
            .service(backup)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))
    .expect("Could not bind the http server on port 8080")
    .run()
    .await
}
