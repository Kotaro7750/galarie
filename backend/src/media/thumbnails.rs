use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use image::{DynamicImage, ImageFormat, ImageReader, imageops::FilterType};
use serde::{Deserialize, Serialize};
use tokio::{process::Command, task, time::timeout};

use crate::indexer::MediaType;

#[allow(dead_code)]
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);
#[allow(dead_code)]
const THUMBNAIL_ROOT: &str = "thumbnails";
#[allow(dead_code)]
const THUMBNAIL_EXT: &str = ".jpg";

/// Default thumbnail sizes supported by the backend.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThumbnailSize {
    Small,
    Medium,
    Large,
}

impl ThumbnailSize {
    pub fn as_dimensions(self) -> (u32, u32) {
        match self {
            ThumbnailSize::Small => (160, 160),
            ThumbnailSize::Medium => (320, 320),
            ThumbnailSize::Large => (640, 640),
        }
    }

    pub fn as_dir(self) -> &'static str {
        match self {
            ThumbnailSize::Small => "small",
            ThumbnailSize::Medium => "medium",
            ThumbnailSize::Large => "large",
        }
    }
}

/// Describes the thumbnail artifact generated for a media file.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailArtifact {
    /// Path relative to the cache directory.
    pub relative_path: PathBuf,
    pub media_type: &'static str,
    pub width: u32,
    pub height: u32,
}

/// Input describing the media file to generate a thumbnail for.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThumbnailSpec {
    pub media_id: String,
    pub source_path: PathBuf,
    pub media_type: MediaType,
}

/// Coordinates on-disk thumbnail generation for images, GIFs, and videos.
#[allow(dead_code)]
pub struct ThumbnailGenerator {
    cache_dir: PathBuf,
    ffmpeg_path: PathBuf,
    gifsicle_path: PathBuf,
    timeout: Duration,
}

#[allow(dead_code)]
impl ThumbnailGenerator {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            gifsicle_path: PathBuf::from("gifsicle"),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_tools(
        mut self,
        ffmpeg_path: impl Into<PathBuf>,
        gifsicle_path: impl Into<PathBuf>,
    ) -> Self {
        self.ffmpeg_path = ffmpeg_path.into();
        self.gifsicle_path = gifsicle_path.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Ensure a thumbnail exists on disk, generating it if missing. Returns the artifact metadata.
    pub async fn ensure_thumbnail(
        &self,
        spec: &ThumbnailSpec,
        size: ThumbnailSize,
    ) -> Result<ThumbnailArtifact> {
        let (target_path, relative_path) = self.thumbnail_paths(&spec.media_id, size);
        if tokio::fs::try_exists(&target_path).await? {
            return Ok(ThumbnailArtifact {
                relative_path,
                media_type: "image/jpeg",
                width: size.as_dimensions().0,
                height: size.as_dimensions().1,
            });
        }

        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        match spec.media_type {
            MediaType::Image | MediaType::Pdf => {
                self.generate_static_thumbnail(&spec.source_path, &target_path, size)
                    .await?;
            }
            MediaType::Gif => {
                self.generate_gif_thumbnail(&spec.source_path, &target_path, size)
                    .await?;
            }
            MediaType::Video => {
                self.generate_video_thumbnail(&spec.source_path, &target_path, size)
                    .await?;
            }
            _ => {
                // fallback to static thumbnail logic
                self.generate_static_thumbnail(&spec.source_path, &target_path, size)
                    .await?;
            }
        }

        Ok(ThumbnailArtifact {
            relative_path,
            media_type: "image/jpeg",
            width: size.as_dimensions().0,
            height: size.as_dimensions().1,
        })
    }

    fn thumbnail_paths(&self, media_id: &str, size: ThumbnailSize) -> (PathBuf, PathBuf) {
        let relative = PathBuf::from(THUMBNAIL_ROOT)
            .join(size.as_dir())
            .join(format!("{media_id}{THUMBNAIL_EXT}"));
        (self.cache_dir.join(&relative), relative)
    }

    async fn generate_static_thumbnail(
        &self,
        source: &Path,
        target: &Path,
        size: ThumbnailSize,
    ) -> Result<()> {
        let source = source.to_owned();
        let target = target.to_owned();
        let (width, height) = size.as_dimensions();
        task::spawn_blocking(move || -> Result<()> {
            let reader = ImageReader::open(&source)
                .and_then(|r| r.with_guessed_format())
                .with_context(|| format!("failed to open image {source:?}"))?;
            let img = reader.decode().context("failed to decode image")?;
            let resized = resize_image(img, width, height);
            save_as_jpeg(resized, &target)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    async fn generate_gif_thumbnail(
        &self,
        source: &Path,
        target: &Path,
        size: ThumbnailSize,
    ) -> Result<()> {
        let (width, height) = size.as_dimensions();
        let output_tmp = target.with_extension("gif.tmp");
        let status = timeout(
            self.timeout,
            Command::new(&self.gifsicle_path)
                .arg("--resize-fit")
                .arg(format!("{width}x{height}"))
                .arg("--no-warnings")
                .arg(source)
                .arg("--output")
                .arg(&output_tmp)
                .status(),
        )
        .await
        .context("gifsicle timed out")??;
        if !status.success() {
            anyhow::bail!("gifsicle failed to process {:?}", source);
        }
        // Convert the GIF output to JPEG for consistency.
        self.generate_static_thumbnail(&output_tmp, target, size)
            .await?;
        tokio::fs::remove_file(output_tmp).await.ok();
        Ok(())
    }

    async fn generate_video_thumbnail(
        &self,
        source: &Path,
        target: &Path,
        size: ThumbnailSize,
    ) -> Result<()> {
        let (width, height) = size.as_dimensions();
        let scale_filter = format!(
            "scale=w={width}:h={height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2"
        );
        let tmp_path = target.with_extension("tmp.jpg");
        let status = timeout(
            self.timeout,
            Command::new(&self.ffmpeg_path)
                .arg("-hide_banner")
                .arg("-loglevel")
                .arg("error")
                .arg("-y")
                .arg("-i")
                .arg(source)
                .arg("-frames:v")
                .arg("1")
                .arg("-vf")
                .arg(&scale_filter)
                .arg(&tmp_path)
                .status(),
        )
        .await
        .context("ffmpeg timed out")??;

        if !status.success() {
            anyhow::bail!("ffmpeg failed to generate poster frame for {:?}", source);
        }

        tokio::fs::rename(&tmp_path, target).await?;
        Ok(())
    }
}

#[allow(dead_code)]
fn resize_image(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
    img.resize(width, height, FilterType::CatmullRom)
}

#[allow(dead_code)]
fn save_as_jpeg(image: DynamicImage, target: &Path) -> Result<()> {
    image
        .save_with_format(target, ImageFormat::Jpeg)
        .context("failed to write jpeg thumbnail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageReader;
    use tempfile::tempdir;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../sample-media")
            .join(name)
    }

    fn assert_thumbnail(path: &Path, size: ThumbnailSize) -> Result<()> {
        let img = ImageReader::open(path)
            .and_then(|r| r.with_guessed_format())?
            .decode()?;
        let (target_w, target_h) = size.as_dimensions();
        assert!(img.width() <= target_w);
        assert!(img.height() <= target_h);
        assert!(img.width() > 0 && img.height() > 0);
        Ok(())
    }

    fn find_tool(tool: &str) -> Option<PathBuf> {
        which::which(tool).ok()
    }

    #[tokio::test]
    async fn generates_thumbnail_for_png() -> Result<()> {
        let dir = tempdir()?;
        let generator = ThumbnailGenerator::new(dir.path()).with_timeout(Duration::from_secs(10));
        let source = fixture("sunset_coast+location-okinawa_rating-5.png");
        let spec = ThumbnailSpec {
            media_id: "png-fixture".into(),
            source_path: source,
            media_type: MediaType::Image,
        };
        let artifact = generator
            .ensure_thumbnail(&spec, ThumbnailSize::Small)
            .await?;
        let final_path = dir.path().join(&artifact.relative_path);
        assert!(tokio::fs::try_exists(&final_path).await?);
        assert_thumbnail(&final_path, ThumbnailSize::Small)?;
        Ok(())
    }

    #[tokio::test]
    async fn generates_thumbnail_for_gif_with_real_gifsicle() -> Result<()> {
        let Some(gifsicle_path) = find_tool("gifsicle") else {
            eprintln!("skipping GIF thumbnail test because gifsicle is not installed");
            return Ok(());
        };

        let dir = tempdir()?;
        let generator = ThumbnailGenerator::new(dir.path())
            .with_tools("ffmpeg", gifsicle_path)
            .with_timeout(Duration::from_secs(10));
        let source = fixture("macro_leaf+subject-nature_rating-4.gif");
        let spec = ThumbnailSpec {
            media_id: "gif-fixture".into(),
            source_path: source,
            media_type: MediaType::Gif,
        };
        let artifact = generator
            .ensure_thumbnail(&spec, ThumbnailSize::Medium)
            .await?;
        let final_path = dir.path().join(&artifact.relative_path);
        assert!(tokio::fs::try_exists(&final_path).await?);
        assert_thumbnail(&final_path, ThumbnailSize::Medium)?;
        Ok(())
    }

    #[tokio::test]
    async fn generates_thumbnail_for_video_with_real_ffmpeg() -> Result<()> {
        let Some(ffmpeg_path) = find_tool("ffmpeg") else {
            eprintln!("skipping video thumbnail test because ffmpeg is not installed");
            return Ok(());
        };

        let dir = tempdir()?;
        let generator = ThumbnailGenerator::new(dir.path())
            .with_tools(ffmpeg_path, "gifsicle")
            .with_timeout(Duration::from_secs(10));
        let source = fixture("skate_session+type-video_rating-3.mp4");
        let spec = ThumbnailSpec {
            media_id: "video-fixture".into(),
            source_path: source,
            media_type: MediaType::Video,
        };
        let artifact = generator
            .ensure_thumbnail(&spec, ThumbnailSize::Large)
            .await?;
        let final_path = dir.path().join(&artifact.relative_path);
        assert!(tokio::fs::try_exists(&final_path).await?);
        assert_thumbnail(&final_path, ThumbnailSize::Large)?;
        Ok(())
    }
}
