#[cfg(test)]
mod testing {
    use crate::{
        configuration::{Configuration, Internationalization, ServerInformation},
        database::{Database, ShortcutEntry},
        health,
        routes::*,
        schema::PutShortcutAnwser,
    };
    use actix_web::{
        http::{
            header::{self, HeaderValue},
            StatusCode,
        },
        test::{self, TestRequest},
        web::Data,
        App,
    };
    use serde_json::json;
    use std::{sync::Arc, fs::{read_dir, read_to_string, remove_dir_all}};

    #[actix_web::test]
    async fn healthcheck() {
        let app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::default().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn admin_dashboard() {
        let data = Arc::new(Database::new(false));
        let tera = tera::Tera::new("./templates/**/*").unwrap();
        // Setup with Basic Authorization protection
        let config = Configuration {
            auth: crate::configuration::Authentication::BasicPrerendered {
                header: "Basic dXNlcm5hbWU6cGFzc3dvcmQ=".to_string(),
            },
            i18n: Internationalization::default(),
            server: ServerInformation::default(),
        };
        let app = test::init_service(
            App::new()
                .app_data(Data::new(config))
                .app_data(Data::new(data))
                .app_data(Data::new(tera))
                .service(dashboard),
        )
        .await;

        // Check if the website is provided
        let mut req = test::TestRequest::default().uri("/").to_request();
        req.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_str("Basic dXNlcm5hbWU6cGFzc3dvcmQ=").unwrap(),
        );
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[actix_web::test]
    async fn url_shortening() {
        let initial_target_uri = "https://github.com";
        let tera = tera::Tera::new("./templates/**/*").unwrap();
        let data = Arc::new(Database::new(false));
        let config = Configuration {
            auth: crate::configuration::Authentication::None,
            i18n: Internationalization::default(),
            server: ServerInformation::default(),
        };

        // Put the URL into the database to be fetched
        let app = test::init_service(
            App::new()
                .app_data(Data::new(config))
                .app_data(Data::new(data))
                .app_data(Data::new(tera))
                .service(create)
                .service(find)
                .service(delete),
        )
        .await;

        let interaction = TestRequest::put()
            .uri("/s")
            .set_json(json!({
                "url": initial_target_uri,
                "slug": "gh",
                "approval": false,
                "since": 0_u128,
                "until": 253370764861000_u128
            }))
            .send_request(&app)
            .await;
        assert_eq!(interaction.response().status(), StatusCode::CREATED);
        assert_eq!(
            interaction
                .response()
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap(),
            "application/json"
        );

        // Get redirection with the newly created slug
        let anwser: PutShortcutAnwser = test::read_body_json(interaction).await;
        let interaction = TestRequest::get()
            .uri(format!("/s/{}", anwser.slug).as_str())
            .send_request(&app)
            .await;
        assert_eq!(interaction.response().status(), StatusCode::SEE_OTHER);
        assert_eq!(
            interaction
                .response()
                .headers()
                .get(header::LOCATION)
                .unwrap(),
            initial_target_uri
        );

        // Delete test entry
        let interaction = TestRequest::delete()
            .uri("/s")
            .set_json(json!({ "slug": anwser.slug }))
            .send_request(&app)
            .await;
        assert_eq!(interaction.response().status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn backups() {
        let database = Database::new(true);

        let slug: String = "garmata".into();
        let value = ShortcutEntry {
            slug: slug.clone(),
            url: "https://github.com/litvinav/garmata".into(),
            status: "trusted".into(),
            since: "0".into(),
            until: "253370761200000".into(),
        };

        database.upsert(slug, value);
        drop(database);

        let dir_entries = read_dir("./backups");
        let entry = dir_entries.unwrap().nth(0).unwrap().unwrap();
        let content = read_to_string(entry.path()).unwrap();
        assert_eq!(content, "garmata,https://github.com/litvinav/garmata,trusted,0,253370761200000");

        remove_dir_all("./backups").expect("could not cleanup backups after test");
    }
}
