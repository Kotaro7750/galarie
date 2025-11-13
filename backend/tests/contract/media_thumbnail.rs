use super::support::{StubApp, response_bytes, response_json};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header::CONTENT_TYPE},
};

#[tokio::test]
async fn thumbnail_returns_binary_payload_with_contract_headers() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media/img-001/thumbnail?size=small")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/png");
    assert!(response.headers().get("ETag").is_some());

    let bytes = response_bytes(response).await;
    assert!(!bytes.is_empty(), "thumbnail body should not be empty");
}

#[tokio::test]
async fn thumbnail_missing_resource_uses_error_envelope() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media/missing-thumb/thumbnail")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = response_json(response).await;
    assert_eq!(json["error"]["code"], "RESOURCE_NOT_FOUND");
}
