use actix_web::http::header::HeaderValue;
use base64::{
    alphabet,
    engine::{general_purpose, Engine as _, GeneralPurpose},
};
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum AuthenticationOptions {
    none,
    basic { username: String, password: String },
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Authentication {
    None,
    Basic { header: String },
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Untrusted {
    pub label: String,
    pub button: String,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Internationalization {
    pub lang: String,
    pub dir: String,
    pub countdown: String,
    pub blocker: String,
    pub approval: Untrusted,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ConfigurationFile {
    #[serde(with = "serde_yaml::with::singleton_map")]
    pub auth: AuthenticationOptions,
    pub i18n: Internationalization,
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Configuration {
    pub auth: Authentication,
    pub i18n: Internationalization,
}

pub fn is_authorized(config: &Configuration, headervalue: Option<&HeaderValue>) -> bool {
    match &config.auth {
        Authentication::None => true,
        Authentication::Basic { header } => {
            if let Some(val) = headervalue {
                return header == val.to_str().unwrap_or_default();
            }
            false
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
            }
        }
    }
}
