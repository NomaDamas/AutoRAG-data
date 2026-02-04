use image::imageops::FilterType;
use image::ImageFormat;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Thumbnail size (max width or height)
const THUMBNAIL_SIZE: u32 = 200;
/// Preview size (max width or height)
const PREVIEW_SIZE: u32 = 1200;

/// Cache manager for storing page thumbnails and previews.
/// Generates WebP thumbnails and previews from database bytea images.
/// Cache is organized by database name and uses chunk_id as the filename.
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(cache_dir.join("thumbnails"))?;
        fs::create_dir_all(cache_dir.join("previews"))?;
        Ok(Self { cache_dir })
    }

    pub fn thumbnail_path(&self, db_name: &str, chunk_id: &i64) -> PathBuf {
        self.cache_dir
            .join("thumbnails")
            .join(db_name)
            .join(format!("{}.webp", chunk_id))
    }

    pub fn preview_path(&self, db_name: &str, chunk_id: &i64) -> PathBuf {
        self.cache_dir
            .join("previews")
            .join(db_name)
            .join(format!("{}.webp", chunk_id))
    }

    pub fn has_thumbnail(&self, db_name: &str, chunk_id: &i64) -> bool {
        self.thumbnail_path(db_name, chunk_id).exists()
    }

    pub fn has_preview(&self, db_name: &str, chunk_id: &i64) -> bool {
        self.preview_path(db_name, chunk_id).exists()
    }

    /// Generate a thumbnail from image bytes (from database bytea column)
    pub fn generate_thumbnail_from_bytes(
        &self,
        image_bytes: &[u8],
        db_name: &str,
        chunk_id: &i64,
    ) -> Result<PathBuf> {
        let thumbnail_path = self.thumbnail_path(db_name, chunk_id);

        // Skip if already exists
        if thumbnail_path.exists() {
            return Ok(thumbnail_path);
        }

        // Ensure db-specific directory exists
        if let Some(parent) = thumbnail_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Decode the image
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| std::io::Error::other(format!("Failed to decode image: {}", e)))?;

        // Resize to thumbnail size
        let thumbnail = img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Lanczos3);

        // Save as WebP
        thumbnail
            .save_with_format(&thumbnail_path, ImageFormat::WebP)
            .map_err(|e| std::io::Error::other(format!("Failed to save thumbnail: {}", e)))?;

        Ok(thumbnail_path)
    }

    /// Generate a preview from image bytes (from database bytea column)
    pub fn generate_preview_from_bytes(
        &self,
        image_bytes: &[u8],
        db_name: &str,
        chunk_id: &i64,
    ) -> Result<PathBuf> {
        let preview_path = self.preview_path(db_name, chunk_id);

        // Skip if already exists
        if preview_path.exists() {
            return Ok(preview_path);
        }

        // Ensure db-specific directory exists
        if let Some(parent) = preview_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Decode the image
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| std::io::Error::other(format!("Failed to decode image: {}", e)))?;

        // Resize to preview size (preserve aspect ratio)
        let preview = img.resize(PREVIEW_SIZE, PREVIEW_SIZE, FilterType::Lanczos3);

        // Save as WebP with high quality
        preview
            .save_with_format(&preview_path, ImageFormat::WebP)
            .map_err(|e| std::io::Error::other(format!("Failed to save preview: {}", e)))?;

        Ok(preview_path)
    }

    /// Clear cache for a specific database
    pub fn clear_db_cache(&self, db_name: &str) -> Result<()> {
        let thumb_dir = self.cache_dir.join("thumbnails").join(db_name);
        if thumb_dir.exists() {
            fs::remove_dir_all(&thumb_dir)?;
        }
        let preview_dir = self.cache_dir.join("previews").join(db_name);
        if preview_dir.exists() {
            fs::remove_dir_all(&preview_dir)?;
        }
        Ok(())
    }

    /// Clear all caches
    pub fn clear_cache(&self) -> Result<()> {
        let thumbnails_dir = self.cache_dir.join("thumbnails");
        if thumbnails_dir.exists() {
            fs::remove_dir_all(&thumbnails_dir)?;
            fs::create_dir_all(&thumbnails_dir)?;
        }

        let previews_dir = self.cache_dir.join("previews");
        if previews_dir.exists() {
            fs::remove_dir_all(&previews_dir)?;
            fs::create_dir_all(&previews_dir)?;
        }

        Ok(())
    }

    pub fn get_cache_size(&self) -> Result<u64> {
        let mut size = 0u64;
        if self.cache_dir.exists() {
            size = Self::dir_size(&self.cache_dir)?;
        }
        Ok(size)
    }

    fn dir_size(path: &Path) -> Result<u64> {
        let mut size = 0u64;
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_dir() {
                    size += Self::dir_size(&entry.path())?;
                } else {
                    size += metadata.len();
                }
            }
        }
        Ok(size)
    }

    /// Get path to cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}
