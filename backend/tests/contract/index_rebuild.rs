use super::support::{response_json, StubApp};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};

#[tokio::test]
async fn rebuild_endpoint_accepts_contract_payload() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/index/rebuild")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{ "force": true }"#))
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let json = response_json(response).await;
    assert_eq!(json["status"], "queued");
    assert!(json.get("startedAt").is_some());
}

#[tokio::test]
async fn rebuild_endpoint_rejects_malformed_json() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/index/rebuild")
        .header("Content-Type", "application/json")
        .body(Body::from("not-json"))
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = response_json(response).await;
    assert_eq!(json["error"]["code"], "VALIDATION_FAILED");
}
