use actix_web::{http::header::HeaderMap, HttpResponse};
use base64::{
    alphabet,
    engine::{general_purpose, Engine as _, GeneralPurpose},
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum AuthenticationOptions {
    none,
    basic {
        username: String,
        password: String,
    },
    oauth2 {
        client_id: String,
        client_secret: String,
        auth_url: String,
        scope: String,
        token_url: String,
        introspect_url: String,
        redirect_url: String,
    },
}

#[derive(Clone)]
pub enum Authentication {
    None,
    Basic {
        header: String,
    },
    OAuth2 {
        client_id: String,
        client_secret: String,
        scope: String,
        auth_url: String,
        token_url: String,
        introspect_url: String,
        redirect_url: String,
    },
}

#[derive(Deserialize, Clone, Default)]
pub struct Untrusted {
    pub label: String,
    pub button: String,
}

#[derive(Deserialize, Clone, Default)]
pub struct Internationalization {
    pub lang: String,
    pub dir: String,
    pub countdown: String,
    pub blocker: String,
    pub approval: Untrusted,
}

#[derive(Deserialize, Clone, Default)]
pub struct ServerInformation {
    pub public_origin: String,
}

#[derive(Deserialize, Clone)]
pub struct ConfigurationFile {
    #[serde(with = "serde_yaml::with::singleton_map")]
    pub auth: AuthenticationOptions,
    pub i18n: Internationalization,
    pub server: ServerInformation,
}

#[derive(Clone)]
pub struct Configuration {
    pub auth: Authentication,
    pub i18n: Internationalization,
    pub server: ServerInformation,
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
                if let Ok(response) = reqwest::Client::new()
                    .get(introspect_url)
                    .bearer_auth(token)
                    .header("Accept", "application/vnd.github+json")
                    .header("User-Agent", "")
                    .send()
                    .await
                {
                    if response.status().as_u16() < 300 {
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
        Authentication::Basic { header } => {
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
        }
    }
}

pub fn get_config() -> Configuration {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/configuration.yaml");
    let filereader = std::fs::File::open(path).expect("missing configuration!");
    let config: ConfigurationFile =
        serde_yaml::from_reader(&filereader).expect("unparsable configuration!");

    if std::env::var("DELETE_CONFIG").unwrap_or_else(|_| "false".to_string()) == *"true" {
        std::fs::remove_file(path).expect("could not delete configuration after loading!");
        log::info!("Removed the configuration.yaml.");
    }

    match config.auth {
        AuthenticationOptions::none => Configuration {
            auth: Authentication::None,
            i18n: config.i18n,
            server: config.server,
        },
        AuthenticationOptions::basic { username, password } => {
            // Prerender Basic Auth header and just compare at runtime
            const ENGINE: GeneralPurpose =
                GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::PAD);
            let b64 = ENGINE.encode(format!("{username}:{password}"));

            Configuration {
                auth: Authentication::Basic {
                    header: format!("Basic {b64}"),
                },
                i18n: config.i18n,
                server: config.server,
            }
        }
        AuthenticationOptions::oauth2 {
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
