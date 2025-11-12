use super::support::{response_json, StubApp};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};

#[tokio::test]
async fn search_returns_contract_shape() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?tags=sunset,travel&page=2&pageSize=2")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = response_json(response).await;
    assert!(json["items"].is_array());
    assert!(json["items"][0]["tags"].is_array());
    assert!(json["items"][0]["attributes"].is_object());
    assert_eq!(json["page"], 2);
    assert_eq!(json["pageSize"], 2);
    assert_eq!(json["total"], 2);
}

#[tokio::test]
async fn search_validation_errors_surface_contract_envelope() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?page=0")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = response_json(response).await;
    assert_eq!(json["error"]["code"], "VALIDATION_FAILED");
    assert!(json["error"]["message"].as_str().unwrap().contains("tags"));
}
