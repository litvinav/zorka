use crate::schema::{
    DeleteShortcut, DeleteShortcutAnwser, GetShortcut, PutShortcut, PutShortcutAnwser,
};
use actix_web::{
    delete, get, put,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use sqlx::{Pool, Row, Sqlite};

#[get("/shortcut/{slug}")]
pub async fn find(data: Data<Pool<Sqlite>>, body: Path<GetShortcut>) -> impl Responder {
    match sqlx::query("SELECT url FROM shortcut WHERE slug = ?;")
        .bind(body.slug.as_str())
        .fetch_one(data.as_ref())
        .await
    {
        Ok(result) => {
            let url: String = result.try_get("url").unwrap();
            HttpResponse::SeeOther()
                .append_header(("Location", url))
                .finish()
        }
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[put("/shortcut")]
pub async fn create(data: Data<Pool<Sqlite>>, body: Json<PutShortcut>) -> impl Responder {
    match sqlx::query("INSERT OR REPLACE INTO shortcut(slug, url) VALUES($1,$2) RETURNING slug;")
        .bind(&body.slug.as_str())
        .bind(&body.url.as_str())
        .fetch_one(data.as_ref())
        .await
    {
        Ok(response) => {
            let slug: String = response.try_get("slug").unwrap();
            log::debug!("Put a new url '{}' with the slug '{}'.", &body.url, &slug);
            HttpResponse::Created().json(PutShortcutAnwser { slug })
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/shortcut")]
pub async fn delete(data: Data<Pool<Sqlite>>, body: Json<DeleteShortcut>) -> impl Responder {
    match sqlx::query("DELETE FROM shortcut WHERE slug = $1 OR url = $1;")
        .bind(&body.text.as_str())
        .execute(data.as_ref())
        .await
    {
        Ok(response) => {
            log::debug!("Deleted {} entries.", response.rows_affected());
            HttpResponse::Ok().json(DeleteShortcutAnwser {
                rows_affected: response.rows_affected(),
            })
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok()
}
