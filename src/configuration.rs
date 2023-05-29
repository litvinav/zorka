use std::process::Command;

use actix_web::{http::header::HeaderMap, HttpResponse};
use base64::{
    alphabet,
    engine::{general_purpose, Engine as _, GeneralPurpose},
};
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize, Default)]
pub struct Configuration {
    pub auth: Authentication,
    pub i18n: Internationalization,
    pub server: ServerInformation,
}

#[derive(Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Authentication {
    #[default]
    None,
    BasicPrerendered {
        header: String,
    },
    Basic {
        username: String,
        password: String,
    },
    OAuth2 {
        client_id: String,
        client_secret: String,
        auth_url: String,
        scope: String,
        token_url: String,
        introspect_url: String,
        redirect_url: String,
    },
}

#[derive(Clone, Deserialize, Default)]
pub struct Internationalization {
    pub lang: String,
    pub dir: String,
    pub countdown: String,
    pub blocker: String,
    pub approval: Untrusted,
}

#[derive(Clone, Deserialize, Default)]
pub struct Untrusted {
    pub label: String,
    pub button: String,
}

#[derive(Clone, Deserialize, Default)]
pub struct ServerInformation {
    pub public_origin: String,
}

#[derive(Deserialize)]
pub struct Oauth2Code {
    pub code: String,
}

pub async fn handle_authorization(
    config: &Configuration,
    headermap: &HeaderMap,
) -> Option<HttpResponse> {
    match &config.auth {
        Authentication::None => None,
        Authentication::OAuth2 {
            client_id,
            scope,
            auth_url,
            redirect_url,
            introspect_url,
            ..
        } => {
            if let Some(value) = headermap.get("Cookie") {
                let token = &value.to_str().unwrap().to_string()[6..];
                if Regex::new(r"^[a-zA-Z0-9-._~]+$").unwrap().is_match(token) {
                    if let Ok(_response) = Command::new("/usr/bin/curl")
                        .arg("-XPOST")
                        .arg("--fail")
                        .args(["-H", "Content-Length: 0"])
                        .args(["-H", "Accept: */*"])
                        .args(["--oauth2-bearer", token])
                        .arg(introspect_url)
                        .output()
                    {
                        return None;
                    }
                }
            }

            Some(
                HttpResponse::SeeOther()
                    .insert_header((
                        "Location",
                        format!("{auth_url}?client_id={client_id}&scope={scope}&response_type=code&redirect_uri={redirect_url}"),
                    ))
                    .finish(),
            )
        }
        Authentication::BasicPrerendered { header } => {
            if let Some(val) = headermap.get("Authorization") {
                if header == val.to_str().unwrap_or_default() {
                    return None;
                }
            }
            Some(
                HttpResponse::Unauthorized()
                    .insert_header(("WWW-Authenticate", "Basic realm=\"Zorka\""))
                    .finish(),
            )
        },
        Authentication::Basic {password: _, username: _} => unimplemented!("runtime lookup of basic auth is not implemented")
    }
}

pub fn get_config() -> Configuration {
    let filereader = std::fs::File::open("./configuration.yaml").expect("missing configuration!");
    let config: Configuration =
        serde_yaml::from_reader(&filereader).expect("unparsable configuration!");

    match config.auth {
        Authentication::None => Configuration {
            auth: Authentication::None,
            i18n: config.i18n,
            server: config.server,
        },
        Authentication::Basic { username, password } => {
            // Prerender Basic Auth header and just compare at runtime
            const ENGINE: GeneralPurpose =
                GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::PAD);
            let b64 = ENGINE.encode(format!("{username}:{password}"));

            Configuration {
                auth: Authentication::BasicPrerendered { header: format!("Basic {b64}") },
                i18n: config.i18n,
                server: config.server,
            }
        }
        Authentication::BasicPrerendered { header } => {
            Configuration {
                auth: Authentication::BasicPrerendered { header },
                i18n: config.i18n,
                server: config.server,
            }
        }
        Authentication::OAuth2 {
            client_id,
            client_secret,
            scope,
            auth_url,
            token_url,
            redirect_url,
            introspect_url,
        } => Configuration {
            auth: Authentication::OAuth2 {
                client_id,
                client_secret,
                scope,
                auth_url,
                token_url,
                redirect_url,
                introspect_url,
            },
            i18n: config.i18n,
            server: config.server,
        },
    }
}
