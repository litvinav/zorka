#[cfg(test)]
mod testing {
    use std::{thread, time::Duration};

    use crate::{
        authentication::{AuthConfig, AuthType, Configuration},
        database::setup_database,
        health,
        routes::*,
        schema::{DeleteShortcutAnwser, PutShortcutAnwser},
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
    use sqlx::{Pool, Sqlite};

    #[actix_web::test]
    async fn healthcheck() {
        let app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::default().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn admin_dashboard() {
        let test_database_path = "/tmp/admin_dashboard.db";
        let pool: Pool<Sqlite> = setup_database(format!("sqlite://{}", test_database_path)).await;
        let tera = tera::Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        // Setup with Basic Authorization protection
        let config = Configuration {
            auth: Some(AuthConfig {
                kind: AuthType::Basic,
                header: "Basic dXNlcm5hbWU6cGFzc3dvcmQ=".to_string(),
            }),
        };
        let app = test::init_service(
            App::new()
                .app_data(Data::new(config.clone()))
                .app_data(Data::new(pool.clone()))
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

        // Cleanup the testing database
        std::fs::remove_file(test_database_path).expect("Testing database could not be deleted");
    }

    #[actix_web::test]
    async fn url_shortening() {
        let initial_target_uri = "https://github.com";
        let test_database_path = "/tmp/url_shortening.db";
        let pool: Pool<Sqlite> = setup_database(format!("sqlite://{}", test_database_path)).await;
        let tera = tera::Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        let config = Configuration { auth: None };

        // Put the URL into the database to be fetched
        let app = test::init_service(
            App::new()
                .app_data(Data::new(config.clone()))
                .app_data(Data::new(pool.clone()))
                .app_data(Data::new(tera.clone()))
                .service(create)
                .service(find)
                .service(delete),
        )
        .await;
        let interaction = TestRequest::put()
            .uri("/s")
            .set_json(json!({ "url": initial_target_uri, "slug": "gh" }))
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

        // May return 404 in GET immediately after PUT during DB write
        thread::sleep(Duration::from_secs(1));

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
            .set_json(json!({ "text": initial_target_uri }))
            .send_request(&app)
            .await;
        assert_eq!(interaction.response().status(), StatusCode::OK);
        let anwser: DeleteShortcutAnwser = test::read_body_json(interaction).await;
        assert_eq!(anwser.rows_affected, 1);

        // Cleanup the testing database
        std::fs::remove_file(test_database_path).expect("Testing database could not be deleted");
    }
}
