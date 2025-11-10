use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, task::JoinHandle, time};
use walkdir::{DirEntry, WalkDir};

/// Representation of a media file discovered on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaFile {
    pub id: String,
    pub relative_path: String,
    pub media_type: MediaType,
    pub tags: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub filesize: u64,
    pub dimensions: Option<Dimensions>,
    pub duration_ms: Option<u64>,
    pub thumbnail_path: Option<String>,
    pub hash: Option<String>,
    pub indexed_at: DateTime<Utc>,
}

/// Placeholder for image/video dimensions. Populated once metadata extraction lands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// Supported media types. `Unknown` is used internally until richer detection ships.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Gif,
    Video,
    Audio,
    Pdf,
    Unknown,
}

impl Default for MediaType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Snapshot + error events emitted by the indexer loop.
#[derive(Debug)]
pub enum IndexEvent {
    Snapshot {
        files: Vec<MediaFile>,
        scanned_at: DateTime<Utc>,
        duration: Duration,
    },
    Error {
        message: String,
    },
}

/// Configuration for the polling-based filesystem watcher.
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub root: PathBuf,
    pub poll_interval: Duration,
}

impl IndexerConfig {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            poll_interval: Duration::from_secs(30),
        }
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
}

/// Handle to the background indexer task.
pub struct IndexerHandle {
    join_handle: JoinHandle<()>,
}

impl IndexerHandle {
    /// Abort the background task without waiting for completion.
    pub fn abort(self) {
        self.join_handle.abort();
    }

    /// Await graceful completion of the indexer loop.
    pub async fn shutdown(self) -> Result<(), tokio::task::JoinError> {
        self.join_handle.await
    }
}

/// Filesystem indexer that periodically scans the media root.
pub struct Indexer;

impl Indexer {
    /// Spawn the polling loop on the Tokio runtime.
    pub fn spawn(config: IndexerConfig) -> (IndexerHandle, mpsc::Receiver<IndexEvent>) {
        let (tx, rx) = mpsc::channel(4);
        let handle = tokio::spawn(async move {
            if let Err(err) = run_loop(config, tx).await {
                tracing::error!(error = ?err, "indexer loop terminated with error");
            }
        });
        (
            IndexerHandle {
                join_handle: handle,
            },
            rx,
        )
    }

    /// Run a one-off filesystem scan (useful for tests or manual rebuilds).
    pub fn scan_once(root: impl AsRef<Path>) -> Result<Vec<MediaFile>> {
        scan_media(root.as_ref())
    }
}

async fn run_loop(config: IndexerConfig, mut tx: mpsc::Sender<IndexEvent>) -> Result<()> {
    emit_snapshot(&config, &mut tx).await?;
    let mut interval = time::interval(config.poll_interval);

    loop {
        interval.tick().await;
        if tx.is_closed() {
            break;
        }
        if let Err(err) = emit_snapshot(&config, &mut tx).await {
            let _ = tx
                .send(IndexEvent::Error {
                    message: err.to_string(),
                })
                .await;
        }
    }

    Ok(())
}

async fn emit_snapshot(config: &IndexerConfig, tx: &mut mpsc::Sender<IndexEvent>) -> Result<()> {
    let root = config.root.clone();
    let started = Instant::now();
    let files = tokio::task::spawn_blocking(move || scan_media(&root)).await??;
    let duration = started.elapsed();
    let event = IndexEvent::Snapshot {
        files,
        scanned_at: Utc::now(),
        duration,
    };
    let _ = tx.send(event).await;
    Ok(())
}

fn scan_media(root: &Path) -> Result<Vec<MediaFile>> {
    if !root.exists() {
        anyhow::bail!(
            "media root '{}' does not exist",
            root.as_os_str().to_string_lossy()
        );
    }

    let mut files = Vec::new();
    let indexed_at = Utc::now();

    for entry in WalkDir::new(root).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                tracing::warn!(error = %err, "failed to read directory entry");
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        match build_media_file(root, &entry, indexed_at) {
            Ok(media_file) => files.push(media_file),
            Err(err) => {
                tracing::warn!(
                    path = %entry.path().display(),
                    error = ?err,
                    "skipping media file due to error"
                );
            }
        }
    }

    Ok(files)
}

fn build_media_file(root: &Path, entry: &DirEntry, indexed_at: DateTime<Utc>) -> Result<MediaFile> {
    let relative = entry
        .path()
        .strip_prefix(root)
        .context("entry not under media root")?;

    let relative_path = relative_to_string(relative);
    let metadata = entry.metadata().context("failed to read metadata")?;
    let filesize = metadata.len();

    Ok(MediaFile {
        id: stable_id(relative),
        relative_path,
        media_type: detect_media_type(entry.path()),
        tags: Vec::new(),
        attributes: HashMap::new(),
        filesize,
        dimensions: None,
        duration_ms: None,
        thumbnail_path: None,
        hash: None,
        indexed_at,
    })
}

fn detect_media_type(path: &Path) -> MediaType {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return MediaType::Unknown;
    };

    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "heic" | "tiff" => MediaType::Image,
        "gif" => MediaType::Gif,
        "mp4" | "mov" | "mkv" | "webm" | "avi" => MediaType::Video,
        "mp3" | "wav" | "flac" | "aac" | "ogg" => MediaType::Audio,
        "pdf" => MediaType::Pdf,
        _ => MediaType::Unknown,
    }
}

fn stable_id(relative: &Path) -> String {
    use sha1::{Digest, Sha1};

    let normalized = relative_to_string(relative);
    let mut hasher = Sha1::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn relative_to_string(path: &Path) -> String {
    let mut normalized = path.to_string_lossy().to_string();
    if std::path::MAIN_SEPARATOR != '/' {
        normalized = normalized.replace(std::path::MAIN_SEPARATOR, "/");
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::time::timeout;

    #[tokio::test]
    async fn scan_once_discovers_files() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();
        std::fs::create_dir_all(root.join("nested"))?;
        std::fs::write(root.join("nested/example.jpg"), b"hello")?;

        let files = Indexer::scan_once(root)?;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, "nested/example.jpg");
        assert_eq!(files[0].media_type, MediaType::Image);
        Ok(())
    }

    #[tokio::test]
    async fn spawn_emits_snapshot_events() -> Result<()> {
        let dir = tempdir()?;
        std::fs::write(dir.path().join("foo.gif"), b"bytes")?;

        let (handle, mut rx) = Indexer::spawn(
            IndexerConfig::new(dir.path()).with_poll_interval(Duration::from_millis(10)),
        );

        let event = timeout(Duration::from_secs(1), rx.recv())
            .await?
            .ok_or_else(|| anyhow!("indexer channel closed"))?;
        match event {
            IndexEvent::Snapshot { files, .. } => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0].media_type, MediaType::Gif);
            }
            IndexEvent::Error { .. } => panic!("expected snapshot"),
        }

        handle.abort();
        Ok(())
    }
}
