#[cfg(test)]
mod tests {
    use crate::health;
    use actix_web::{http::StatusCode, test, App};

    #[actix_web::test]
    async fn healthcheck() {
        let app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::default().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
