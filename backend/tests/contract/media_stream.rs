use super::support::{response_bytes, response_json, StubApp};
use axum::{
    body::Body,
    http::{
        header::{ACCEPT_RANGES, CONTENT_RANGE},
        Method, Request, StatusCode,
    },
};

#[tokio::test]
async fn stream_supports_partial_content_requests() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media/vid-001/stream?disposition=inline")
        .header("Range", "bytes=0-11")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        response.headers().get(ACCEPT_RANGES).unwrap(),
        "bytes"
    );
    assert_eq!(
        response.headers().get(CONTENT_RANGE).unwrap(),
        "bytes 0-11/12"
    );

    let bytes = response_bytes(response).await;
    assert_eq!(bytes.len(), 12);
}

#[tokio::test]
async fn stream_invalid_disposition_returns_validation_error() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media/vid-001/stream?disposition=download")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = response_json(response).await;
    assert_eq!(json["error"]["code"], "VALIDATION_FAILED");
}
