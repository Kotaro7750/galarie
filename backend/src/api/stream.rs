use std::{
    cmp,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Path as PathParam, Query, State},
    http::{
        HeaderMap, StatusCode,
        header::{
            ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
        },
    },
    response::Response,
};
use mime_guess::MimeGuess;
use serde::Deserialize;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncSeekExt},
};
use tokio_util::io::ReaderStream;

use crate::{
    api::{ApiError, ErrorCode},
    indexer::{MediaFile, MediaType},
    routes::AppState,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamParams {
    pub disposition: Option<String>,
}

pub async fn media_stream(
    PathParam(media_id): PathParam<String>,
    Query(params): Query<StreamParams>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let disposition = params
        .disposition
        .as_deref()
        .unwrap_or("inline")
        .to_lowercase();
    if disposition != "inline" && disposition != "attachment" {
        return Err(ApiError::bad_request(
            "disposition must be inline or attachment",
        ));
    }

    let media = {
        let snapshot = state.snapshot.read().await;
        snapshot
            .media
            .iter()
            .find(|item| item.id == media_id)
            .cloned()
    }
    .ok_or_else(|| ApiError::not_found("media not found"))?;

    let absolute_path = resolve_media_path(&state.config.media_root, &media.relative_path).await?;
    let metadata = fs::metadata(&absolute_path)
        .await
        .map_err(ApiError::internal_with_source)?;
    if !metadata.is_file() {
        return Err(ApiError::not_found("media not found"));
    }

    let file_size = metadata.len();
    let range_header = headers
        .get(axum::http::header::RANGE)
        .and_then(|value| value.to_str().ok());
    let range = parse_range(range_header, file_size)?;

    let mut file = fs::File::open(&absolute_path)
        .await
        .map_err(ApiError::internal_with_source)?;

    let (status, body_length, body_stream) = match range {
        StreamRange::Full => {
            let stream = ReaderStream::new(file);
            (StatusCode::OK, file_size, Body::from_stream(stream))
        }
        StreamRange::Partial { start, end } => {
            let len = end - start + 1;
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(ApiError::internal_with_source)?;
            let limited = file.take(len);
            let stream = ReaderStream::new(limited);
            (StatusCode::PARTIAL_CONTENT, len, Body::from_stream(stream))
        }
    };

    let content_type = derive_content_type(&media, &absolute_path);
    let file_name = Path::new(&media.relative_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("media");
    let content_disposition = format!("{disposition}; filename=\"{file_name}\"");
    let etag = format!("\"{}-{}\"", media.id, file_size);

    let mut response = Response::builder()
        .status(status)
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_DISPOSITION, content_disposition)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_LENGTH, body_length.to_string())
        .header(ETAG, etag);

    if let StreamRange::Partial { start, end } = range {
        response = response.header(CONTENT_RANGE, format!("bytes {start}-{end}/{file_size}"));
    }

    response
        .body(body_stream)
        .map_err(|err| ApiError::internal_with_source(anyhow!(err)))
}

async fn resolve_media_path(root: &Path, relative: &str) -> Result<PathBuf, ApiError> {
    let root = root.to_path_buf();
    let root_canonical = fs::canonicalize(&root)
        .await
        .map_err(ApiError::internal_with_source)?;
    let candidate = root.join(relative);
    let candidate_canonical = match fs::canonicalize(&candidate).await {
        Ok(path) => path,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(ApiError::not_found("media not found"));
        }
        Err(err) => return Err(ApiError::internal_with_source(err)),
    };

    if !candidate_canonical.starts_with(&root_canonical) {
        return Err(ApiError::forbidden(
            "access outside media root is not allowed",
        ));
    }

    Ok(candidate_canonical)
}

fn derive_content_type(media: &MediaFile, path: &Path) -> String {
    if let Some(guess) = MimeGuess::from_path(path).first_raw() {
        return guess.to_string();
    }

    match media.media_type {
        MediaType::Image => "image/jpeg".into(),
        MediaType::Gif => "image/gif".into(),
        MediaType::Video => "video/mp4".into(),
        MediaType::Audio => "audio/mpeg".into(),
        MediaType::Pdf => "application/pdf".into(),
        MediaType::Unknown => "application/octet-stream".into(),
    }
}

#[derive(Clone, Copy, Debug)]
enum StreamRange {
    Full,
    Partial { start: u64, end: u64 },
}

fn parse_range(range_header: Option<&str>, total: u64) -> Result<StreamRange, ApiError> {
    let Some(value) = range_header else {
        return Ok(StreamRange::Full);
    };

    if !value.starts_with("bytes=") {
        return Err(ApiError::bad_request("range must be expressed in bytes"));
    }

    let spec = &value[6..];
    if spec.contains(',') {
        return Err(ApiError::bad_request("multiple ranges are not supported"));
    }

    let (start, end) = if let Some(rest) = spec.strip_prefix('-') {
        let suffix: u64 = rest
            .parse()
            .map_err(|_| ApiError::bad_request("invalid range suffix"))?;
        if suffix == 0 {
            return Err(ApiError::bad_request("invalid range suffix"));
        }
        let suffix = cmp::min(suffix, total);
        (total - suffix, total - 1)
    } else {
        let mut parts = spec.splitn(2, '-');
        let start_str = parts.next().unwrap_or_default();
        let end_str = parts.next().unwrap_or_default();
        if start_str.is_empty() {
            return Err(ApiError::bad_request("range start is required"));
        }
        let start: u64 = start_str
            .parse()
            .map_err(|_| ApiError::bad_request("invalid range start"))?;
        if start >= total {
            return Err(ApiError::with_status(
                StatusCode::RANGE_NOT_SATISFIABLE,
                ErrorCode::ValidationFailed,
                "range start exceeds file length",
            ));
        }
        let end = if end_str.is_empty() {
            total - 1
        } else {
            let parsed_end: u64 = end_str
                .parse()
                .map_err(|_| ApiError::bad_request("invalid range end"))?;
            parsed_end
        };
        if end < start {
            return Err(ApiError::bad_request("range end must be >= start"));
        }
        let capped_end = cmp::min(end, total - 1);
        (start, capped_end)
    };

    Ok(StreamRange::Partial { start, end })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_suffix_range() {
        let range = parse_range(Some("bytes=-500"), 1_000).expect("range");
        match range {
            StreamRange::Partial { start, end } => {
                assert_eq!(start, 500);
                assert_eq!(end, 999);
            }
            _ => panic!("expected partial range"),
        }
    }

    #[test]
    fn parses_open_ended_range() {
        let range = parse_range(Some("bytes=250-"), 1_000).expect("range");
        match range {
            StreamRange::Partial { start, end } => {
                assert_eq!(start, 250);
                assert_eq!(end, 999);
            }
            _ => panic!("expected partial range"),
        }
    }

    #[test]
    fn rejects_out_of_bounds_start() {
        let err = parse_range(Some("bytes=2000-"), 1_000).unwrap_err();
        assert_eq!(err.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    }
}
