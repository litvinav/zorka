#[cfg(test)]
mod testing {
    use std::collections::HashMap;

    use crate::{
        configuration::{Configuration, Internationalization, ServerInformation},
        database::Database,
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

    #[actix_web::test]
    async fn healthcheck() {
        let app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::default().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn admin_dashboard() {
        let data = Database::new(HashMap::new());
        let tera = tera::Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        // Setup with Basic Authorization protection
        let config = Configuration {
            auth: crate::configuration::Authentication::Basic {
                header: "Basic dXNlcm5hbWU6cGFzc3dvcmQ=".to_string(),
            },
            i18n: Internationalization::default(),
            server: ServerInformation::default(),
        };
        let app = test::init_service(
            App::new()
                .app_data(Data::new(config.clone()))
                .app_data(Data::new(data.clone()))
                .app_data(Data::new(tera.clone()))
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
        let data = Database::new(HashMap::new());
        let config = Configuration {
            auth: crate::configuration::Authentication::None,
            i18n: Internationalization::default(),
            server: ServerInformation::default()
        };

        // Put the URL into the database to be fetched
        let app = test::init_service(
            App::new()
                .app_data(Data::new(config.clone()))
                .app_data(Data::new(data.clone()))
                .app_data(Data::new(tera.clone()))
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

        // // May return 404 in GET immediately after PUT during DB write
        // thread::sleep(Duration::from_secs(1));

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
}
