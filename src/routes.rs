use crate::{configuration::*, schema::*};
use actix_files::NamedFile;
use actix_web::{
    delete, get,
    http::header,
    put,
    web::{Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};
use regex::Regex;
use sqlx::{Pool, Row, Sqlite};
use std::time::{SystemTime, UNIX_EPOCH};
use tera::{Context, Tera};

#[get("/")]
pub async fn dashboard(
    data: Data<Pool<Sqlite>>,
    tera: Data<Tera>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if !is_authorized(config.as_ref(), req.headers().get("Authorization")) {
        return HttpResponse::Unauthorized()
            .insert_header(("WWW-Authenticate", "Basic realm=\"Zorka\""))
            .finish();
    }

    match sqlx::query("SELECT slug,url,since,until,status FROM shortcut ORDER BY slug;")
        .fetch_all(data.as_ref())
        .await
    {
        Ok(result) => {
            let items: Vec<ShortcutItem> = result
                .iter()
                .map(|item| ShortcutItem {
                    slug: item.get("slug"),
                    url: item.get("url"),
                    status: item.get("status"),
                    now: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("time went backwards")
                        .as_millis(),
                    since: item
                        .get::<String, &str>("since")
                        .parse()
                        .expect("not valid UNIX time."),
                    until: item
                        .get::<String, &str>("until")
                        .parse()
                        .expect("not valid UNIX time."),
                })
                .collect();
            match tera.render(
                "dashboard.html",
                &Context::from_serialize(ShortcutList { items }).expect(""),
            ) {
                Ok(html) => HttpResponse::Ok()
                    .insert_header(header::ContentType::html())
                    .body(html),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[get("/s")]
pub async fn render(
    data: Data<Pool<Sqlite>>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if !is_authorized(config.as_ref(), req.headers().get("Authorization")) {
        return HttpResponse::Unauthorized()
            .insert_header(("WWW-Authenticate", "Basic realm=\"Zorka\""))
            .finish();
    }

    let csv = match sqlx::query("SELECT slug,url,status,since,until FROM shortcut;")
        .fetch_all(data.as_ref())
        .await
    {
        Ok(result) => result
            .iter()
            .map(|row| {
                let slug: String = row.get("slug");
                let url: String = row.get("url");
                let status: String = row.get("status");
                let since: String = row.get("since");
                let until: String = row.get("until");
                format!("{slug},{url},{status},{since},{until}")
            })
            .collect::<Vec<String>>()
            .join("\n"),
        Err(_) => String::default(),
    };
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/csv; charset utf-8"))
        .append_header(("Content-Disposition", "attachment; filename=\"seed.csv\""))
        .body(csv)
}

#[get("/s/{slug}")]
pub async fn find(
    data: Data<Pool<Sqlite>>,
    tera: Data<Tera>,
    config: Data<Configuration>,
    path: Path<GetShortcut>,
) -> impl Responder {
    match sqlx::query("SELECT url,since,until,status FROM shortcut WHERE slug = ?;")
        .bind(path.slug.as_str())
        .fetch_one(data.as_ref())
        .await
    {
        Ok(result) => {
            let url: String = result.get("url");
            let status: String = result.get("status");
            let available_since: u128 = result
                .get::<String, &str>("since")
                .parse()
                .expect("not valid UNIX time.");
            let available_until: u128 = result
                .get::<String, &str>("until")
                .parse()
                .expect("not valid UNIX time.");
            let now: u128 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_millis();

            if now >= available_since && now <= available_until {
                if status == "untrusted" {
                    let ctx = Context::from_serialize(Approval {
                        url,
                        dir: config.i18n.dir.clone(),
                        lang: config.i18n.lang.clone(),
                        label: config.i18n.approval.label.clone(),
                        button: config.i18n.approval.button.clone(),
                    }).unwrap();
                    if let Ok(html) = tera.render("gate/approval.html", &ctx) {
                         return HttpResponse::Ok()
                                .insert_header(header::ContentType::html())
                                .body(html);
                    }
                } else {
                    return HttpResponse::SeeOther()
                    .append_header(("Location", url))
                    .finish();
                }
            } else if now < available_since {
                let ctx = Context::from_serialize(Countdown {
                    timestamp: available_since,
                    dir: config.i18n.dir.clone(),
                    lang: config.i18n.lang.clone(),
                    label: config.i18n.countdown.clone(),
                }).unwrap();
                if let Ok(html) = tera.render("gate/countdown.html",&ctx) {
                    return HttpResponse::Ok()
                        .insert_header(header::ContentType::html())
                        .body(html);
                };
            } else {
                let ctx = Context::from_serialize(Blocker {
                    dir: config.i18n.dir.clone(),
                    lang: config.i18n.lang.clone(),
                    label: config.i18n.blocker.clone(),
                }).unwrap();
                if let Ok(html) =  tera.render("gate/blocker.html", &ctx) {
                    return HttpResponse::Ok()
                    .insert_header(header::ContentType::html())
                    .body(html);
                }
            }
            HttpResponse::InternalServerError().finish()
        }
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[put("/s")]
pub async fn create(
    data: Data<Pool<Sqlite>>,
    body: Json<PutShortcut>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if !is_authorized(config.as_ref(), req.headers().get("Authorization")) {
        return HttpResponse::Unauthorized()
            .insert_header(("WWW-Authenticate", "Basic realm=\"Zorka\""))
            .finish();
    }

    // Validation
    if body.slug.len() > 64 || body.slug.is_empty() {
        return HttpResponse::UnprocessableEntity().body("Provide a non empty slug (max. 64).");
    }

    let regex =
        Regex::new(r"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()!@:%_\+.~#?&//=]*)$")
        .expect("invalid url regex");
    if regex.captures(&body.url).is_none() {
        return HttpResponse::UnprocessableEntity().body("The provided URL is invalid.");
    }

    // Insert
    match sqlx::query(
        "INSERT OR REPLACE INTO shortcut(slug,url,status,since,until) VALUES($1,$2,$3,$4,$5) RETURNING slug;",
    )
    .bind(body.slug.as_str())
    .bind(body.url.as_str())
    .bind(if body.approval {"untrusted"} else {"trusted"})
    .bind(body.since.to_string())
    .bind(body.until.to_string())
    .fetch_one(data.as_ref())
    .await
    {
        Ok(response) => {
            let slug: String = response.get("slug");
            log::debug!("Put a new slug '{}'.", &slug);
            HttpResponse::Created().json(PutShortcutAnwser { slug })
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/s")]
pub async fn delete(
    data: Data<Pool<Sqlite>>,
    body: Json<DeleteShortcut>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if !is_authorized(config.as_ref(), req.headers().get("Authorization")) {
        return HttpResponse::Unauthorized()
            .insert_header(("WWW-Authenticate", "Basic realm=\"Zorka\""))
            .finish();
    }
    match sqlx::query("DELETE FROM shortcut WHERE slug = $1 OR url = $1;")
        .bind(body.text.as_str())
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

#[get("/assets/{file:.*}")]
async fn assets(req: HttpRequest, path: Path<Assets>) -> HttpResponse {
    match NamedFile::open(format!(
        "{}/assets/{}",
        env!("CARGO_MANIFEST_DIR"),
        path.file
    )) {
        Ok(file) => {
            let mut res = file.into_response(&req);
            res.headers_mut().append(
                header::CACHE_CONTROL,
                header::HeaderValue::from_str("max-age=360")
                    .expect("couldn't create cache header."),
            );
            res
        }
        Err(err) => {
            println!("Error opening {}: {}", path.file, err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
