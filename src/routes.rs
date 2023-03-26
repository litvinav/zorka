use crate::{configuration::*, database::Database, schema::*};
use actix_web::{
    delete, get,
    http::header,
    put,
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use base64::{
    alphabet,
    engine::{general_purpose, Engine as _, GeneralPurpose},
};
use qrcode::{render::svg, EcLevel, QrCode, Version};
use regex::Regex;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tera::{Context, Tera};

#[get("/")]
pub async fn dashboard(
    data: Data<Database>,
    tera: Data<Tera>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if let Some(res) = handle_authorization(config.as_ref(), req.headers()).await {
        return res;
    }

    let items = data
        .read_all()
        .iter()
        .map(|item| ShortcutItem {
            slug: item.slug.clone(),
            url: item.url.clone(),
            status: item.status.clone(),
            now: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_millis(),
            since: item.since.parse().expect("not valid UNIX time."),
            until: item.until.parse().expect("not valid UNIX time."),
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

#[get("/s")]
pub async fn render(
    data: Data<Database>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if let Some(res) = handle_authorization(config.as_ref(), req.headers()).await {
        return res;
    }

    let csv = data
        .read_all()
        .iter()
        .map(|row| {
            format!(
                "{},{},{},{},{}",
                row.slug, row.url, row.status, row.since, row.until,
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/csv; charset utf-8"))
        .append_header(("Content-Disposition", "attachment; filename=\"seed.csv\""))
        .body(csv)
}

#[get("/s/{slug}")]
pub async fn find(
    data: Data<Database>,
    tera: Data<Tera>,
    config: Data<Configuration>,
    path: Path<GetShortcut>,
) -> impl Responder {
    match data.read(&path.slug) {
        Some(result) => {
            let available_since: u128 = result.since.parse().expect("not valid UNIX time.");
            let available_until: u128 = result.until.parse().expect("not valid UNIX time.");
            let now: u128 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_millis();

            if now >= available_since && now <= available_until {
                // Approval confirm url
                if result.status == "untrusted" {
                    let ctx = Context::from_serialize(Approval {
                        url: result.url,
                        dir: config.i18n.dir.clone(),
                        lang: config.i18n.lang.clone(),
                        label: config.i18n.approval.label.clone(),
                        button: config.i18n.approval.button.clone(),
                    })
                    .unwrap();
                    if let Ok(html) = tera.render("gate/approval.html", &ctx) {
                        return HttpResponse::Ok()
                            .insert_header(header::ContentType::html())
                            .body(html);
                    }
                }
                // Redirect
                else {
                    return HttpResponse::SeeOther()
                        .append_header(("Location", result.url))
                        .finish();
                }
            } else if now < available_since {
                // Countdown
                let ctx = Context::from_serialize(Countdown {
                    timestamp: available_since,
                    dir: config.i18n.dir.clone(),
                    lang: config.i18n.lang.clone(),
                    label: config.i18n.countdown.clone(),
                })
                .unwrap();
                if let Ok(html) = tera.render("gate/countdown.html", &ctx) {
                    return HttpResponse::Ok()
                        .insert_header(header::ContentType::html())
                        .body(html);
                };
            } else {
                // Block outdated
                let ctx = Context::from_serialize(Blocker {
                    dir: config.i18n.dir.clone(),
                    lang: config.i18n.lang.clone(),
                    label: config.i18n.blocker.clone(),
                })
                .unwrap();
                if let Ok(html) = tera.render("gate/blocker.html", &ctx) {
                    return HttpResponse::Ok()
                        .insert_header(header::ContentType::html())
                        .body(html);
                }
            }
            // Unreachable 500
            HttpResponse::InternalServerError().finish()
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/share/{slug}")]
pub async fn share(
    path: Path<GetShortcut>,
    config: Data<Configuration>,
    tera: Data<Tera>,
    req: HttpRequest,
) -> impl Responder {
    if let Some(res) = handle_authorization(config.as_ref(), req.headers()).await {
        return res;
    }

    let url = format!("{}/s/{}", config.server.public_origin, path.slug);
    let qrcode = QrCode::with_version(url.as_bytes(), Version::Normal(4), EcLevel::L).unwrap();
    let vector = qrcode
        .render()
        .quiet_zone(false)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("transparent"))
        .min_dimensions(300, 300)
        .build();
    const ENGINE: GeneralPurpose =
        GeneralPurpose::new(&alphabet::STANDARD, general_purpose::NO_PAD);

    match tera.render(
        "share.html",
        &Context::from_serialize(Share {
            slug: path.slug.to_string(),
            vector: ENGINE.encode(vector),
        })
        .unwrap(),
    ) {
        Ok(html) => HttpResponse::Ok()
            .insert_header(header::ContentType::html())
            .body(html),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[put("/s")]
pub async fn create(
    data: Data<Database>,
    body: Json<PutShortcut>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if let Some(res) = handle_authorization(config.as_ref(), req.headers()).await {
        return res;
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
    let status = if body.approval {
        "untrusted"
    } else {
        "trusted"
    };
    let entry = ShortcutEntry {
        slug: body.slug.clone(),
        url: body.url.clone(),
        status: status.to_string(),
        since: body.since.to_string(),
        until: body.until.to_string(),
    };

    if data.upsert(body.slug.clone(), entry) {
        log::debug!("Put a new slug '{}'.", &body.slug);
        HttpResponse::Created().json(PutShortcutAnwser {
            slug: body.slug.clone(),
        })
    } else {
        HttpResponse::InternalServerError().body("Please retry again in a few seconds.")
    }
}

#[delete("/s")]
pub async fn delete(
    data: Data<Database>,
    body: Json<DeleteShortcut>,
    config: Data<Configuration>,
    req: HttpRequest,
) -> impl Responder {
    if let Some(res) = handle_authorization(config.as_ref(), req.headers()).await {
        return res;
    }

    if data.delete(&body.slug) {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().body("Please retry again in a few seconds.")
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/oauth2/code")]
pub async fn code(config: Data<Configuration>, query: Query<Oauth2Code>) -> impl Responder {
    if let Authentication::OAuth2 {
        client_id,
        client_secret,
        token_url,
        redirect_url,
        ..
    } = &config.auth
    {
        let answer: serde_json::Value = reqwest::Client::new()
            .post(format!(
                "{token_url}?grant_type=authorization_code&client_id={client_id}&redirect_uri={redirect_url}&client_secret={client_secret}&code={}",
                query.code
            ))
            .header("Content-Length", "0")
            .header("Accept", "application/json")
            .send()
            .await
            .expect("failed to get a response")
            .json()
            .await
            .expect("failed to get a payload");

        if let Some(Value::String(token)) = answer.get("access_token") {
            return HttpResponse::Ok()
            .insert_header(("Set-Cookie", format!("token={token}; Path=/; HttpOnly; Secure; SameSite=None")))
            .insert_header(("Content-Type", "text/html"))
            .body("<!DOCTYPE html><html><head><meta http-equiv=\"refresh\" content=\"0; url='/'\"></head><body></body></html>");
        }
    }

    HttpResponse::InternalServerError().finish()
}
