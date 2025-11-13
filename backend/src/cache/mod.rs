use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::indexer::MediaFile;

const CACHE_VERSION: &str = "1.0.0";
const CACHE_FILENAME: &str = "index.json";

/// Snapshot of indexed media persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheSnapshot {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub media: Vec<MediaFile>,
}

impl CacheSnapshot {
    pub fn new(media: Vec<MediaFile>) -> Self {
        Self {
            version: CACHE_VERSION.to_string(),
            generated_at: Utc::now(),
            media,
        }
    }
}

/// JSON cache store that manages read/write lifecycle for the index snapshot.
#[derive(Debug)]
pub struct CacheStore {
    path: PathBuf,
}

impl CacheStore {
    /// Create a new store rooted at the provided cache directory.
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let mut path = cache_dir.into();
        path.push(CACHE_FILENAME);
        Self { path }
    }

    /// Load the cache from disk if present and compatible with the current schema version.
    pub fn load(&self) -> Result<Option<CacheSnapshot>> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                let snapshot: CacheSnapshot =
                    serde_json::from_str(&contents).context("failed to parse cache json")?;
                if snapshot.version != CACHE_VERSION {
                    anyhow::bail!(
                        "cache schema mismatch (found {}, expected {})",
                        snapshot.version,
                        CACHE_VERSION
                    );
                }
                Ok(Some(snapshot))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Persist the provided media list to disk, returning the snapshot that was written.
    pub fn persist(&self, media: Vec<MediaFile>) -> Result<CacheSnapshot> {
        let snapshot = CacheSnapshot::new(media);
        self.write_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    /// Attempt to load an existing cache, falling back to a rebuild if none or invalid.
    pub fn load_or_rebuild<F>(&self, rebuild: F) -> Result<CacheSnapshot>
    where
        F: FnOnce() -> Result<Vec<MediaFile>>,
    {
        match self.load() {
            Ok(Some(snapshot)) => Ok(snapshot),
            Ok(None) => {
                tracing::info!("cache missing, triggering rebuild");
                self.rebuild_with(rebuild)
            }
            Err(err) => {
                tracing::warn!(error = %err, "failed to read cache, rebuilding");
                self.rebuild_with(rebuild)
            }
        }
    }

    fn rebuild_with<F>(&self, rebuild: F) -> Result<CacheSnapshot>
    where
        F: FnOnce() -> Result<Vec<MediaFile>>,
    {
        let media = rebuild().context("rebuild callback failed")?;
        self.persist(media)
    }

    fn write_snapshot(&self, snapshot: &CacheSnapshot) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = self.path.with_extension(format!(
            "{}.tmp",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let json =
            serde_json::to_string_pretty(snapshot).context("failed to serialize cache snapshot")?;

        fs::write(&tmp_path, json)?;
        fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::MediaType;
    use anyhow::{Result, anyhow};
    use tempfile::tempdir;

    fn sample_media() -> MediaFile {
        MediaFile {
            id: "abc".into(),
            relative_path: "foo/bar.jpg".into(),
            media_type: MediaType::Image,
            tags: vec![],
            attributes: Default::default(),
            filesize: 42,
            dimensions: None,
            duration_ms: None,
            thumbnail_path: Some("/media/abc/thumbnail".into()),
            hash: None,
            indexed_at: Utc::now(),
        }
    }

    #[test]
    fn persist_and_load_roundtrip() -> Result<()> {
        let dir = tempdir()?;
        let store = CacheStore::new(dir.path());
        let written = store.persist(vec![sample_media()])?;
        assert_eq!(written.media.len(), 1);

        let loaded = store.load()?.expect("should load snapshot");
        assert_eq!(loaded.media.len(), 1);
        assert_eq!(loaded.media[0].relative_path, "foo/bar.jpg");
        Ok(())
    }

    #[test]
    fn load_or_rebuild_invokes_fallback_when_missing() -> Result<()> {
        let dir = tempdir()?;
        let store = CacheStore::new(dir.path());
        let snapshot = store.load_or_rebuild(|| Ok(vec![sample_media()]))?;
        assert_eq!(snapshot.media.len(), 1);

        // Subsequent load should reuse cache instead of rebuilding.
        let reused = store.load_or_rebuild(|| Err(anyhow!("should not rebuild")))?;
        assert_eq!(reused.media.len(), 1);
        Ok(())
    }
}
