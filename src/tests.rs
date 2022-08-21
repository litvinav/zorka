#[cfg(test)]
mod tests {
    use crate::{
        database::setup_database,
        health,
        routes::{create, find},
    };
    use actix_web::{
        http::{header, StatusCode},
        test::{self, TestRequest},
        web::Data,
        App,
    };
    use serde::Deserialize;
    use serde_json::json;
    use sqlx::{Pool, Sqlite};

    #[actix_web::test]
    async fn healthcheck() {
        let app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::default().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[derive(Deserialize)]
    struct PutShortcutAnwser {
        slug: String,
    }
    #[actix_web::test]
    async fn url_shortening() {
        let initial_target_uri = "https://www.github.com";
        let test_database_path = "/tmp/testing.db";
        let pool: Pool<Sqlite> = setup_database(format!("sqlite://{}", test_database_path)).await;

        // Put the URL into the database to be fetched
        let app = test::init_service(
            App::new()
                .app_data(Data::new(pool))
                .service(create)
                .service(find),
        )
        .await;
        let interaction = TestRequest::put()
            .uri("/shortcut")
            .set_json(json!({ "url": initial_target_uri }))
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
            .uri(format!("/shortcut/{}", anwser.slug).as_str())
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

        // Clearnup the testing database
        std::fs::remove_file(test_database_path).expect("Testing database could not be deleted");
    }
}
