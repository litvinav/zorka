use actix_web::{
    get, put,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use sqlx::{Pool, Row, Sqlite};

use crate::schema::{GetShortcut, PutShortcut, PutShortcutAnwser};

#[get("/shortcut/{slug}")]
pub async fn find(data: Data<Pool<Sqlite>>, body: Path<GetShortcut>) -> impl Responder {
    log::debug!("Getting '{}'", body.slug.as_str());
    let result = sqlx::query("SELECT url FROM shortcut WHERE slug = ?;")
        .bind(body.slug.as_str())
        .fetch_one(data.as_ref())
        .await;

    if result.is_ok() {
        let url: String = result.unwrap().try_get("url").unwrap();
        log::debug!("Found slug '{}' for url '{}'", body.slug.as_str(), url);
        return HttpResponse::Found()
            .append_header(("Location", url))
            .finish();
    }
    return HttpResponse::NotFound().finish();
}

#[put("/shortcut")]
pub async fn create(data: Data<Pool<Sqlite>>, body: Json<PutShortcut>) -> impl Responder {
    let slug = nanoid::nanoid!();
    log::debug!("Putting '{}' into the database.", &slug);

    let row =
        sqlx::query("INSERT OR REPLACE INTO shortcut(slug, url) VALUES($1,$2) RETURNING slug;")
            .bind(slug)
            .bind(&body.url.as_str())
            .fetch_one(data.as_ref())
            .await;

    match row {
        Ok(response) => {
            let created_slug: String = response.try_get("slug").unwrap();
            log::debug!(
                "Put a new url '{}' with the slug '{}'",
                &body.url,
                &created_slug,
            );
            HttpResponse::Created().json(PutShortcutAnwser { slug: created_slug })
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
