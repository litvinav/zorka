use actix_web::http::header::HeaderValue;

#[derive(Clone, PartialEq, Eq)]
pub enum AuthType {
    Basic,
}
#[derive(Clone)]
pub struct AuthConfig {
    pub kind: AuthType,
    pub header: String,
}
#[derive(Clone)]
pub struct Configuration {
    pub auth: Option<AuthConfig>,
}

pub fn is_authorized(config: &Configuration, header: Option<&HeaderValue>) -> bool {
    if config.auth.is_some() {
        match config.auth.as_ref().unwrap().kind {
            AuthType::Basic => {
                let config_ref = config.auth.as_ref().unwrap();
                match config_ref.kind {
                    AuthType::Basic => {
                        if let Some(value) = header {
                            if let Ok(actual) = value.to_str() {
                                return actual == config_ref.header;
                            }
                        }
                    }
                }
            }
        }
    } else {
        return true;
    }

    false
}

pub fn get_config() -> Configuration {
    if std::env::var("AUTH").is_ok() {
        log::info!("Basic auth enabled.");
        Configuration {
            auth: Some(AuthConfig {
                kind: AuthType::Basic,
                header: std::env::var("AUTH").expect(""),
            }),
        }
    } else {
        Configuration { auth: None }
    }
}
