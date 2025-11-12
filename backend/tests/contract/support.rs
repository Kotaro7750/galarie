use axum::{
    body::Body,
    extract::{
        rejection::{JsonRejection, PathRejection, QueryRejection},
        Json, Path, Query,
    },
    http::{
        header::{
            ACCEPT_RANGES, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
        },
        HeaderMap, HeaderValue, Request, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use http_body_util::BodyExt;
use serde::Deserialize;
use serde_json::{json, Value};
use tower::ServiceExt;

#[derive(Clone)]
pub struct StubApp {
    router: Router,
}

impl StubApp {
    pub fn new() -> Self {
        Self {
            router: build_router(),
        }
    }

    pub async fn request(&self, request: Request<Body>) -> Response {
        self.router
            .clone()
            .oneshot(request)
            .await
            .expect("stub router to respond")
    }
}

pub async fn response_json(response: Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body bytes")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("valid json payload")
}

pub async fn response_bytes(response: Response) -> Bytes {
    response
        .into_body()
        .collect()
        .await
        .expect("body bytes")
        .to_bytes()
}

fn build_router() -> Router {
    Router::new()
        .route("/api/v1/media", get(media_search))
        .route("/api/v1/media/{id}/thumbnail", get(media_thumbnail))
        .route("/api/v1/media/{id}/stream", get(media_stream))
        .route("/api/v1/index/rebuild", post(index_rebuild))
        .fallback(not_found_handler)
}

async fn media_search(query: Result<Query<SearchQuery>, QueryRejection>) -> Response {
    let Query(query) = match query {
        Ok(value) => value,
        Err(_) => {
            return validation_failed("invalid query parameters");
        }
    };

    let tags = query.tags.as_deref().map(str::trim).unwrap_or("");
    if tags.is_empty() {
        return validation_failed("tags query parameter is required");
    }

    let page = query.page.unwrap_or(1);
    if page == 0 {
        return validation_failed("page must be greater than or equal to 1");
    }

    let page_size = query.page_size.unwrap_or(60);
    if page_size == 0 || page_size > 200 {
        return validation_failed("pageSize must be between 1 and 200");
    }

    let payload = json!({
        "items": [
            {
                "id": "img-001",
                "relativePath": "photos/sunsets/img-001.jpg",
                "mediaType": "image",
                "tags": [
                    {
                        "rawToken": "sunset",
                        "type": "simple",
                        "name": "sunset",
                        "value": null,
                        "normalized": "sunset"
                    },
                    {
                        "rawToken": "rating-5",
                        "type": "kv",
                        "name": "rating",
                        "value": "5",
                        "normalized": "rating=5"
                    }
                ],
                "attributes": {
                    "rating": "5",
                    "camera": "alpha"
                },
                "filesize": 24567,
                "dimensions": {
                    "width": 1920,
                    "height": 1080
                },
                "durationMs": null,
                "thumbnailPath": "/thumbnails/img-001.jpg",
                "indexedAt": "2025-01-01T12:00:00Z"
            },
            {
                "id": "gif-002",
                "relativePath": "gifs/loop/gif-002.gif",
                "mediaType": "gif",
                "tags": [
                    {
                        "rawToken": "loop",
                        "type": "simple",
                        "name": "loop",
                        "value": null,
                        "normalized": "loop"
                    }
                ],
                "attributes": {},
                "filesize": 9876,
                "dimensions": {
                    "width": 640,
                    "height": 480
                },
                "durationMs": 3000,
                "thumbnailPath": "/thumbnails/gif-002.gif",
                "indexedAt": "2025-01-01T12:05:00Z"
            }
        ],
        "total": 2,
        "page": page,
        "pageSize": page_size
    });

    (StatusCode::OK, Json(payload)).into_response()
}

async fn media_thumbnail(
    path: Result<Path<String>, PathRejection>,
    query: Result<Query<ThumbnailQuery>, QueryRejection>,
) -> Response {
    let Path(id) = match path {
        Ok(value) => value,
        Err(_) => return validation_failed("invalid media identifier"),
    };

    let Query(query) = match query {
        Ok(value) => value,
        Err(_) => return validation_failed("invalid thumbnail parameters"),
    };

    if let Some(size) = query.size.as_deref() {
        if !matches!(size, "small" | "medium" | "large") {
            return validation_failed("size must be one of small, medium, or large");
        }
    }

    if id == "missing-thumb" {
        return contract_error(
            StatusCode::NOT_FOUND,
            "RESOURCE_NOT_FOUND",
            "thumbnail not available",
        );
    }

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("image/png"))
        .header(ETAG, HeaderValue::from_static("\"stub-thumb-etag\""))
        .header(CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"))
        .body(Body::from(Bytes::from_static(b"\x89PNG\r\nstub-data")))
        .expect("valid thumbnail response");

    response
}

async fn media_stream(
    path: Result<Path<String>, PathRejection>,
    headers: HeaderMap,
    query: Result<Query<StreamQuery>, QueryRejection>,
) -> Response {
    let Path(id) = match path {
        Ok(value) => value,
        Err(_) => return validation_failed("invalid media identifier"),
    };

    let Query(query) = match query {
        Ok(value) => value,
        Err(_) => return validation_failed("invalid stream parameters"),
    };

    if let Some(disposition) = query.disposition.as_deref() {
        if !matches!(disposition, "inline" | "attachment") {
            return validation_failed("disposition must be inline or attachment");
        }
    }

    if id == "missing-stream" {
        return contract_error(
            StatusCode::NOT_FOUND,
            "RESOURCE_NOT_FOUND",
            "media file not found",
        );
    }

    let mut builder = Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"))
        .header(ACCEPT_RANGES, HeaderValue::from_static("bytes"));

    if headers.contains_key("Range") {
        builder = builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header(
                CONTENT_RANGE,
                HeaderValue::from_static("bytes 0-11/12"),
            )
            .header(CONTENT_LENGTH, HeaderValue::from_static("12"));
        builder
            .body(Body::from(Bytes::from_static(b"partial-bytes")))
            .expect("partial stream response")
    } else {
        builder = builder
            .status(StatusCode::OK)
            .header(CONTENT_LENGTH, HeaderValue::from_static("12"));
        builder
            .body(Body::from(Bytes::from_static(b"full-response")))
            .expect("full stream response")
    }
}

async fn index_rebuild(
    payload: Result<Json<IndexRebuildRequest>, JsonRejection>,
) -> Response {
    let Json(payload) = match payload {
        Ok(value) => value,
        Err(_) => return validation_failed("invalid JSON payload"),
    };

    let response = json!({
        "status": "queued",
        "startedAt": "2025-01-01T12:10:00Z",
        "finishedAt": null
    });

    (StatusCode::ACCEPTED, Json(response)).into_response()
}

async fn not_found_handler() -> Response {
    contract_error(StatusCode::NOT_FOUND, "RESOURCE_NOT_FOUND", "route not found")
}

fn validation_failed(message: impl Into<String>) -> Response {
    contract_error(
        StatusCode::BAD_REQUEST,
        "VALIDATION_FAILED",
        message,
    )
}

fn contract_error(
    status: StatusCode,
    code: &'static str,
    message: impl Into<String>,
) -> Response {
    let payload = json!({
        "error": {
            "code": code,
            "message": message.into()
        }
    });

    (status, Json(payload)).into_response()
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    tags: Option<String>,
    page: Option<usize>,
    #[serde(rename = "pageSize")]
    page_size: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ThumbnailQuery {
    size: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamQuery {
    disposition: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IndexRebuildRequest {
    #[serde(default)]
    force: bool,
}
