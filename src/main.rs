use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use crate::routes::{find,create};

mod routes;
mod tests;
mod database;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting HTTP server at http://0:8080");

    let db_filename = 
    std::env::var("DATABASE_URL").expect("DATABASE_URL env is required to target the sqlite database");
    let pools = database::setup_database(db_filename).await;

    HttpServer::new(move || {
        App::new()
            .service(health)
            .app_data(web::Data::new(pools.clone()))
            .service(find)
            .service(create)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))
    .expect("Could not bind the http server on port 8080")
    .run()
    .await
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok()
}
