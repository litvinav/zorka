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
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("zorka=error"));

    let data = database::setup();
    let tera = Tera::new("./templates/**/*").unwrap();
    let config = get_config();

    log::info!("Starting HTTP server at http://127.1:8080");
    HttpServer::new(move || {
        App::new()
            .service(health)
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(data.clone()))
            .app_data(web::Data::new(tera.clone()))
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
