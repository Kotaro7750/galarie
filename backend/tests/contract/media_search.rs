use super::support::{StubApp, response_json};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};

#[tokio::test]
async fn search_applies_and_semantics_for_tags_and_attributes() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?tags=sunset,coast&attributes[rating]=5")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = response_json(response).await;
    let items = json["items"].as_array().expect("items array");
    assert_eq!(
        items.len(),
        1,
        "AND filters should narrow to a single match"
    );
    assert_eq!(items[0]["id"], "img-001");
    assert_eq!(json["total"], 1);
}

#[tokio::test]
async fn search_supports_filterless_browsing_with_pagination() {
    let app = StubApp::new();
    // page 1 without filters should return the first catalog item
    let first = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?page=1&pageSize=1")
        .body(Body::empty())
        .expect("request");
    let first_response = app.request(first).await;
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_json = response_json(first_response).await;
    assert_eq!(first_json["items"][0]["id"], "gif-003");
    assert_eq!(first_json["total"], 3);

    // page 2 should yield the second catalog item even without filters
    let second = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?page=2&pageSize=1")
        .body(Body::empty())
        .expect("request");
    let second_response = app.request(second).await;
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_json = response_json(second_response).await;
    assert_eq!(second_json["items"][0]["id"], "img-001");
    assert_eq!(second_json["page"], 2);
}

#[tokio::test]
async fn search_accepts_kv_tag_name_filters() {
    let app = StubApp::new();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/media?tags=camera")
        .body(Body::empty())
        .expect("request");

    let response = app.request(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let json = response_json(response).await;
    assert_eq!(json["total"], 1);
    assert_eq!(json["items"][0]["id"], "img-001");
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
    assert!(json["error"]["message"].as_str().unwrap().contains("page"));
}
