use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{
    api::{ApiError, ApiResult},
    routes::AppState,
    services::search::{SearchQuery, SearchResult, SearchService},
};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RawSearchParams {
    pub tags: Option<String>,
    pub page: Option<usize>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<usize>,
    #[serde(flatten)]
    pub rest: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSearchResponse {
    pub items: Vec<crate::indexer::MediaFile>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

pub async fn media_search(
    State(state): State<AppState>,
    Query(params): Query<RawSearchParams>,
) -> ApiResult<MediaSearchResponse> {
    let tags = parse_tags(params.tags.as_deref()).map_err(|msg| ApiError::bad_request(msg))?;

    let attributes = parse_attributes(&params.rest);
    let query = SearchQuery::new(
        tags,
        attributes,
        params.page.unwrap_or(1),
        params.page_size.unwrap_or(60),
    );
    let snapshot = state.snapshot.read().await;
    let result = SearchService::search(&snapshot, &query);

    Ok(Json(MediaSearchResponse::from(result)))
}

impl From<SearchResult> for MediaSearchResponse {
    fn from(value: SearchResult) -> Self {
        Self {
            items: value.items,
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}

fn parse_tags(raw: Option<&str>) -> Result<Vec<String>, &'static str> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };

    let tags: Vec<String> = raw
        .split(',')
        .map(|token| token.trim().to_lowercase())
        .filter(|token| !token.is_empty())
        .collect();

    if tags.is_empty() {
        Err("tags query parameter must contain at least one value")
    } else {
        Ok(tags)
    }
}

fn parse_attributes(rest: &HashMap<String, String>) -> HashMap<String, Vec<String>> {
    let mut attributes = HashMap::new();
    for (key, value) in rest {
        if let Some(name) = key
            .strip_prefix("attributes[")
            .and_then(|s| s.strip_suffix(']'))
        {
            let values = value
                .split(',')
                .map(|token| token.trim().to_lowercase())
                .filter(|token| !token.is_empty())
                .collect::<Vec<_>>();
            if !values.is_empty() {
                attributes.insert(name.to_lowercase(), values);
            }
        }
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CacheSnapshot,
        config::{AppConfig, LogConfig, OtelConfig},
        indexer::{MediaFile, MediaType},
        tags::{Tag, TagKind},
    };
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use chrono::Utc;
    use http_body_util::BodyExt;
    use std::{net::SocketAddr, sync::Arc};
    use tempfile::tempdir;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    fn app_state_with_media(media: Vec<MediaFile>) -> AppState {
        let tmp = tempdir().unwrap();
        let config = Arc::new(AppConfig {
            media_root: tmp.path().to_path_buf(),
            cache_dir: tmp.path().to_path_buf(),
            listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            environment: "test".into(),
            otel: OtelConfig {
                endpoint: None,
                service_name: "test".into(),
                disable_traces: true,
                disable_logs: true,
            },
            log: LogConfig {
                level: "info".into(),
            },
            cors_allowed_origins: Vec::new(),
            frontend_dist_dir: None,
        });
        let cache_store = Arc::new(crate::cache::CacheStore::new(tmp.path()));
        let snapshot = CacheSnapshot::new(media);
        AppState::new(config, cache_store, Arc::new(RwLock::new(snapshot)))
    }

    fn sample_media(id: &str, tags: Vec<Tag>) -> MediaFile {
        let mut attributes = HashMap::new();
        for tag in &tags {
            if matches!(tag.kind, TagKind::KeyValue) {
                if let Some(value) = &tag.value {
                    attributes
                        .entry(tag.name.clone())
                        .or_insert_with(|| value.clone());
                }
            }
        }

        MediaFile {
            id: id.to_string(),
            relative_path: format!("{id}.png"),
            media_type: MediaType::Image,
            tags,
            attributes,
            filesize: 0,
            dimensions: None,
            duration_ms: None,
            thumbnail_path: Some(format!("/media/{id}/thumbnail")),
            hash: None,
            indexed_at: Utc::now(),
        }
    }

    fn simple_tag(name: &str) -> Tag {
        Tag {
            raw_token: name.into(),
            kind: TagKind::Simple,
            name: name.to_lowercase(),
            value: None,
            normalized: name.to_lowercase(),
        }
    }

    fn kv_tag(key: &str, value: &str) -> Tag {
        Tag {
            raw_token: format!("{key}-{value}"),
            kind: TagKind::KeyValue,
            name: key.to_lowercase(),
            value: Some(value.to_lowercase()),
            normalized: format!("{}={}", key.to_lowercase(), value.to_lowercase()),
        }
    }

    #[tokio::test]
    async fn allows_browsing_without_filters() {
        let media = vec![
            sample_media(
                "sunset_A",
                vec![simple_tag("sunset"), kv_tag("rating", "5")],
            ),
            sample_media("macro_B", vec![simple_tag("macro"), kv_tag("rating", "4")]),
        ];
        let state = app_state_with_media(media);
        let router = crate::routes::router(state);
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/media?page=1&pageSize=2")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["total"], 2);
        assert_eq!(payload["items"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn returns_matching_media() {
        let media = vec![
            sample_media(
                "sunset_A",
                vec![simple_tag("sunset"), kv_tag("rating", "5")],
            ),
            sample_media("macro_B", vec![simple_tag("macro"), kv_tag("rating", "4")]),
        ];
        let state = app_state_with_media(media);
        let router = crate::routes::router(state);
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/media?tags=sunset&attributes[rating]=5")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["total"], 1);
        assert_eq!(payload["items"][0]["id"], "sunset_A");
    }

    #[tokio::test]
    async fn matches_kv_tag_names_with_tags_query() {
        let media = vec![
            sample_media("camera_A", vec![kv_tag("camera", "alpha")]),
            sample_media("other_B", vec![simple_tag("other")]),
        ];
        let state = app_state_with_media(media);
        let router = crate::routes::router(state);
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/media?tags=camera")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["total"], 1);
        assert_eq!(payload["items"][0]["id"], "camera_A");
    }
}
