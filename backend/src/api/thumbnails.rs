use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, ETAG},
    },
    response::Response,
};
use serde::Deserialize;

use crate::{
    api::ApiError,
    media::thumbnails::{ThumbnailGenerator, ThumbnailSize, ThumbnailSpec},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailParams {
    pub size: Option<ThumbnailSize>,
}

pub async fn media_thumbnail(
    Path(media_id): Path<String>,
    Query(params): Query<ThumbnailParams>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let size = params.size.unwrap_or(ThumbnailSize::Medium);

    let spec = {
        let snapshot = state.snapshot.read().await;
        snapshot
            .media
            .iter()
            .find(|media| media.id == media_id)
            .map(|media| ThumbnailSpec {
                media_id: media.id.clone(),
                source_path: state.config.media_root.join(&media.relative_path),
                media_type: media.media_type.clone(),
            })
    };

    let spec = match spec {
        Some(spec) => spec,
        None => return Err(ApiError::not_found("media not found")),
    };

    let generator = ThumbnailGenerator::new(state.config.cache_dir.clone());
    let artifact = generator
        .ensure_thumbnail(&spec, size)
        .await
        .map_err(ApiError::internal_with_source)?;

    let absolute = state.config.cache_dir.join(&artifact.relative_path);
    let bytes = tokio::fs::read(&absolute)
        .await
        .map_err(ApiError::internal_with_source)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, artifact.media_type)
        .header(CACHE_CONTROL, "public, max-age=3600")
        .header(ETAG, format!("\"{}-{}\"", spec.media_id, size.as_dir()))
        .header(CONTENT_LENGTH, bytes.len().to_string())
        .body(Body::from(bytes))
        .map_err(|err| ApiError::internal_with_source(anyhow!(err)))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CacheSnapshot,
        config::{AppConfig, LogConfig, OtelConfig},
        indexer::{MediaFile, MediaType},
        routes::AppState,
        tags::{Tag, TagKind},
    };
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use chrono::Utc;
    use http_body_util::BodyExt;
    use image::{DynamicImage, ImageBuffer, Rgb};
    use std::{collections::HashMap as Map, net::SocketAddr, sync::Arc};
    use tempfile::tempdir;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_thumbnail_for_existing_media() {
        let tmp = tempdir().unwrap();
        let media_root = tmp.path().join("media");
        tokio::fs::create_dir_all(&media_root).await.unwrap();
        let cache_dir = tmp.path().join("cache");
        tokio::fs::create_dir_all(&cache_dir).await.unwrap();

        let image_path = media_root.join("sample.png");
        save_png(&image_path);

        let media = MediaFile {
            id: "sample".into(),
            relative_path: "sample.png".into(),
            media_type: MediaType::Image,
            tags: vec![simple_tag("sample")],
            attributes: Map::new(),
            filesize: 0,
            dimensions: None,
            duration_ms: None,
            thumbnail_path: Some("/media/sample/thumbnail".into()),
            hash: None,
            indexed_at: Utc::now(),
        };

        let state = app_state(media, media_root, cache_dir);
        let router = crate::routes::router(state);
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/media/sample/thumbnail?size=small")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[CONTENT_TYPE], "image/jpeg");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_media() {
        let tmp = tempdir().unwrap();
        let state = app_state(
            MediaFile {
                id: "sample".into(),
                relative_path: "missing.png".into(),
                media_type: MediaType::Image,
                tags: vec![],
                attributes: Map::new(),
                filesize: 0,
                dimensions: None,
                duration_ms: None,
                thumbnail_path: Some("/media/sample/thumbnail".into()),
                hash: None,
                indexed_at: Utc::now(),
            },
            tmp.path().join("media"),
            tmp.path().join("cache"),
        );
        let router = crate::routes::router(state);
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/media/unknown/thumbnail")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    fn app_state(
        media: MediaFile,
        media_root: std::path::PathBuf,
        cache_dir: std::path::PathBuf,
    ) -> AppState {
        let config = Arc::new(AppConfig {
            media_root,
            cache_dir: cache_dir.clone(),
            listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            environment: "test".into(),
            otel: OtelConfig {
                endpoint: None,
                service_name: "test".into(),
            },
            log: LogConfig {
                level: "info".into(),
            },
            cors_allowed_origins: Vec::new(),
        });
        let cache_store = Arc::new(crate::cache::CacheStore::new(&cache_dir));
        let snapshot = CacheSnapshot::new(vec![media]);
        AppState::new(config, cache_store, Arc::new(RwLock::new(snapshot)))
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

    fn save_png(path: &std::path::Path) {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(10, 10, Rgb([255, 0, 0]));
        DynamicImage::ImageRgb8(img)
            .save(path)
            .expect("failed to create sample png");
    }
}
