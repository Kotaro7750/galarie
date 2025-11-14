use axum::{
    Router,
    body::Body,
    extract::{
        Json, Path, Query,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{
        HeaderMap, HeaderValue, Request, StatusCode,
        header::{
            ACCEPT_RANGES, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE,
            CONTENT_TYPE, ETAG,
        },
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use bytes::Bytes;
use http_body_util::BodyExt;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
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

    let tags = query.required_tags();
    let page = query.page.unwrap_or(1);
    if page == 0 {
        return validation_failed("page must be greater than or equal to 1");
    }

    let page_size = query.page_size.unwrap_or(60);
    if page_size == 0 || page_size > 200 {
        return validation_failed("pageSize must be between 1 and 200");
    }

    let attributes = query.attribute_filters();
    let mut matches: Vec<_> = stub_media()
        .into_iter()
        .filter(|item| item.matches(&tags, &attributes))
        .map(StubMedia::into_value)
        .collect();

    // deterministic order for pagination expectations
    matches.sort_by(|a, b| {
        a.get("id")
            .and_then(Value::as_str)
            .cmp(&b.get("id").and_then(Value::as_str))
    });

    let total = matches.len();
    let start = (page - 1) * page_size;
    let end = start.saturating_add(page_size).min(total);
    let items = if start >= total {
        Vec::new()
    } else {
        matches[start..end].to_vec()
    };

    let payload = json!({
        "items": items,
        "total": total,
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
        .header(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        )
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

    let disposition = query.disposition.as_deref().unwrap_or("inline");
    let content_disposition = format!("{disposition}; filename=\"{id}\"");

    let mut builder = Response::builder()
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        )
        .header(ACCEPT_RANGES, HeaderValue::from_static("bytes"))
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&content_disposition).unwrap(),
        );

    if headers.contains_key("Range") {
        let body = Bytes::from_static(b"partial-byts");
        builder = builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header(CONTENT_RANGE, HeaderValue::from_static("bytes 0-11/12"))
            .header(
                CONTENT_LENGTH,
                HeaderValue::from_str(&body.len().to_string()).unwrap(),
            );
        builder
            .body(Body::from(body))
            .expect("partial stream response")
    } else {
        let body = Bytes::from_static(b"full-response");
        builder = builder.status(StatusCode::OK).header(
            CONTENT_LENGTH,
            HeaderValue::from_str(&body.len().to_string()).unwrap(),
        );
        builder
            .body(Body::from(body))
            .expect("full stream response")
    }
}

async fn index_rebuild(payload: Result<Json<IndexRebuildRequest>, JsonRejection>) -> Response {
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
    contract_error(
        StatusCode::NOT_FOUND,
        "RESOURCE_NOT_FOUND",
        "route not found",
    )
}

fn validation_failed(message: impl Into<String>) -> Response {
    contract_error(StatusCode::BAD_REQUEST, "VALIDATION_FAILED", message)
}

fn contract_error(status: StatusCode, code: &'static str, message: impl Into<String>) -> Response {
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
    #[serde(flatten)]
    other: HashMap<String, String>,
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

#[derive(Clone)]
struct StubMedia {
    id: &'static str,
    relative_path: &'static str,
    media_type: &'static str,
    tags: &'static [&'static str],
    attributes: &'static [(&'static str, &'static str)],
    filesize: u64,
    dimensions: Option<(u32, u32)>,
    duration_ms: Option<u64>,
}

impl SearchQuery {
    fn required_tags(&self) -> Vec<String> {
        self.tags.as_deref().map(parse_csv).unwrap_or_default()
    }

    fn attribute_filters(&self) -> HashMap<String, Vec<String>> {
        let mut filters = HashMap::new();
        for (key, value) in &self.other {
            if let Some(name) = key
                .strip_prefix("attributes[")
                .and_then(|rest| rest.strip_suffix(']'))
            {
                filters
                    .entry(name.to_ascii_lowercase())
                    .or_insert_with(Vec::new)
                    .extend(parse_csv(value).into_iter());
            }
        }
        filters
    }
}

impl StubMedia {
    fn matches(&self, tags: &[String], attributes: &HashMap<String, Vec<String>>) -> bool {
        for tag in tags {
            let has_simple_tag = self
                .tags
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(tag));
            let has_attribute_key = self
                .attributes
                .iter()
                .any(|(key, _)| key.eq_ignore_ascii_case(tag));
            if !(has_simple_tag || has_attribute_key) {
                return false;
            }
        }

        for (key, values) in attributes {
            let value = self
                .attributes
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| *v);

            match value {
                Some(actual) => {
                    if !values
                        .iter()
                        .any(|wanted| actual.eq_ignore_ascii_case(wanted))
                    {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    fn into_value(self) -> Value {
        json!({
            "id": self.id,
            "relativePath": self.relative_path,
            "mediaType": self.media_type,
            "tags": self.tags.iter().map(|tag| json!({
                "rawToken": tag,
                "type": if tag.contains('-') { "keyvalue" } else { "simple" },
                "name": tag.split_once('-').map(|(name, _)| name).unwrap_or(tag),
                "value": tag.split_once('-').map(|(_, value)| value),
                "normalized": tag.to_ascii_lowercase().replace('-', "="),
            })).collect::<Vec<_>>(),
            "attributes": self.attributes.iter().cloned().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<_, _>>(),
            "filesize": self.filesize,
            "dimensions": self.dimensions.map(|(width, height)| json!({ "width": width, "height": height })),
            "durationMs": self.duration_ms,
            "thumbnailPath": format!("/thumbnails/{}.jpg", self.id),
            "indexedAt": "2025-01-01T12:00:00Z"
        })
    }
}

fn stub_media() -> Vec<StubMedia> {
    vec![
        StubMedia {
            id: "img-001",
            relative_path: "photos/sunsets/img-001.jpg",
            media_type: "image",
            tags: &["sunset", "coast", "rating-5"],
            attributes: &[("rating", "5"), ("camera", "alpha")],
            filesize: 24_567,
            dimensions: Some((1920, 1080)),
            duration_ms: None,
        },
        StubMedia {
            id: "img-002",
            relative_path: "photos/sunsets/img-002.jpg",
            media_type: "image",
            tags: &["sunset", "forest", "rating-4"],
            attributes: &[("rating", "4")],
            filesize: 19_000,
            dimensions: Some((1600, 900)),
            duration_ms: None,
        },
        StubMedia {
            id: "gif-003",
            relative_path: "gifs/loop/gif-003.gif",
            media_type: "gif",
            tags: &["loop", "fun"],
            attributes: &[("rating", "3")],
            filesize: 9_876,
            dimensions: Some((640, 480)),
            duration_ms: Some(3_000),
        },
    ]
}

fn parse_csv(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|token| token.trim())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}
