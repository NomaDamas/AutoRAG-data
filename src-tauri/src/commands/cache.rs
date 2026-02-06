use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::State;
use tokio::task::spawn_blocking;

use crate::cache::CacheManager;
use crate::error::{AppError, Result};
use crate::ingest::render_page_to_png;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Source classification
// ---------------------------------------------------------------------------

enum SourceType {
    /// Directly servable image file (png/jpg/webp)
    Image(PathBuf),
    /// PDF — needs per-page rendering via pdftoppm
    Pdf(PathBuf),
    /// No usable source (missing path, missing file, unknown extension)
    None,
}

fn classify_source(source_path: &Option<String>) -> SourceType {
    let path_str = match source_path {
        Some(s) if !s.is_empty() => s,
        _ => return SourceType::None,
    };
    let p = Path::new(path_str);
    if !p.exists() {
        return SourceType::None;
    }
    match p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp") => SourceType::Image(p.to_path_buf()),
        Some("pdf") => SourceType::Pdf(p.to_path_buf()),
        _ => SourceType::None,
    }
}

/// Query source info for a page: page_num and source_path (via page_metadata or file table).
/// Does NOT fetch BYTEA.
struct PageSourceInfo {
    page_num: i32,
    source_path: Option<String>,
}

async fn query_page_source_info(
    pool: &sqlx::PgPool,
    page_id: i64,
) -> Result<PageSourceInfo> {
    let row: (i32, Option<String>) = sqlx::query_as(
        r#"
        SELECT p.page_num,
               COALESCE(p.page_metadata->>'source_path', f.path) as source_path
        FROM page p
        JOIN document d ON p.document_id = d.id
        LEFT JOIN file f ON d.path = f.id
        WHERE p.id = $1
        "#,
    )
    .bind(page_id)
    .fetch_one(pool)
    .await?;

    Ok(PageSourceInfo {
        page_num: row.0,
        source_path: row.1,
    })
}

/// Read image bytes from a source, returning None if source is unavailable.
/// For images, reads the file directly. For PDFs, renders the specific page.
async fn read_bytes_from_source(source: &SourceType, page_num: i32) -> Option<Vec<u8>> {
    match source {
        SourceType::Image(path) => std::fs::read(path).ok(),
        SourceType::Pdf(path) => {
            let path = path.clone();
            spawn_blocking(move || render_page_to_png(&path, page_num).ok())
                .await
                .ok()
                .flatten()
        }
        SourceType::None => None,
    }
}

// ---------------------------------------------------------------------------
// Cache helpers (lock-safe — acquire and release Mutex without holding across await)
// ---------------------------------------------------------------------------

fn check_and_get_original_path(
    cache: &Mutex<Option<CacheManager>>,
    db_name: &str,
    page_id: i64,
) -> Result<(PathBuf, bool)> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    let path = cm.original_path(db_name, &page_id);
    let exists = path.exists();
    Ok((path, exists))
}

fn save_original(
    cache: &Mutex<Option<CacheManager>>,
    bytes: &[u8],
    db_name: &str,
    page_id: i64,
) -> Result<PathBuf> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    cm.save_original(bytes, db_name, &page_id)
}

fn check_and_get_thumbnail_path(
    cache: &Mutex<Option<CacheManager>>,
    db_name: &str,
    chunk_id: i64,
) -> Result<(PathBuf, bool)> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    let path = cm.thumbnail_path(db_name, &chunk_id);
    let exists = path.exists();
    Ok((path, exists))
}

pub(crate) fn generate_thumbnail(
    cache: &Mutex<Option<CacheManager>>,
    image_bytes: &[u8],
    db_name: &str,
    chunk_id: i64,
) -> Result<PathBuf> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    cm.generate_thumbnail_from_bytes(image_bytes, db_name, &chunk_id)
}

fn check_and_get_preview_path(
    cache: &Mutex<Option<CacheManager>>,
    db_name: &str,
    chunk_id: i64,
) -> Result<(PathBuf, bool)> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    let path = cm.preview_path(db_name, &chunk_id);
    let exists = path.exists();
    Ok((path, exists))
}

pub(crate) fn generate_preview(
    cache: &Mutex<Option<CacheManager>>,
    image_bytes: &[u8],
    db_name: &str,
    chunk_id: i64,
) -> Result<PathBuf> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    cm.generate_preview_from_bytes(image_bytes, db_name, &chunk_id)
}

fn has_thumbnail(
    cache: &Mutex<Option<CacheManager>>,
    db_name: &str,
    chunk_id: i64,
) -> Result<bool> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    Ok(cm.has_thumbnail(db_name, &chunk_id))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Get full-resolution page image. Returns a file path (not a data URL).
///
/// Flow:
/// 1. Try source file (image → direct path, PDF → render + cache)
/// 2. Fall back to originals cache
/// 3. Fall back to BYTEA from database
#[tauri::command]
pub async fn get_page_image_url(
    page_id: i64,
    state: State<'_, AppState>,
    cache: State<'_, Mutex<Option<CacheManager>>>,
) -> Result<String> {
    let db_name = state
        .get_db_identifier()
        .await
        .ok_or(AppError::NotConnected)?;
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Query source info (no BYTEA)
    let info = query_page_source_info(&pool, page_id).await?;
    let source = classify_source(&info.source_path);

    // For directly servable images, return the path immediately
    if let SourceType::Image(ref path) = source {
        return Ok(path.to_string_lossy().to_string());
    }

    // For PDF or None, check originals cache first
    let (original_path, cached) = check_and_get_original_path(&cache, &db_name, page_id)?;
    if cached {
        return Ok(original_path.to_string_lossy().to_string());
    }

    // Try rendering from PDF source
    if let Some(bytes) = read_bytes_from_source(&source, info.page_num).await {
        let path = save_original(&cache, &bytes, &db_name, page_id)?;
        return Ok(path.to_string_lossy().to_string());
    }

    // Final fallback: fetch BYTEA from database
    let row: (Option<Vec<u8>>,) = sqlx::query_as(
        "SELECT image_contents FROM page WHERE id = $1",
    )
    .bind(page_id)
    .fetch_one(&pool)
    .await?;

    let contents = row
        .0
        .ok_or_else(|| AppError::NotFound(format!("Page {} has no image contents", page_id)))?;

    let path = save_original(&cache, &contents, &db_name, page_id)?;
    Ok(path.to_string_lossy().to_string())
}

/// Get image chunk as a data URL from bytea column (unchanged — not on hot path)
#[tauri::command]
pub async fn get_chunk_image_url(chunk_id: i64, state: State<'_, AppState>) -> Result<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let row: (Vec<u8>, String) = sqlx::query_as(
        "SELECT contents, mimetype FROM image_chunk WHERE id = $1",
    )
    .bind(chunk_id)
    .fetch_one(&pool)
    .await?;

    let (contents, mimetype) = row;
    let base64_data = STANDARD.encode(&contents);
    Ok(format!("data:{};base64,{}", mimetype, base64_data))
}

/// Get thumbnail URL — cached WebP thumbnail, generated from source file or BYTEA
#[tauri::command]
pub async fn get_thumbnail_url(
    page_id: i64,
    state: State<'_, AppState>,
    cache: State<'_, Mutex<Option<CacheManager>>>,
) -> Result<String> {
    let db_name = state
        .get_db_identifier()
        .await
        .ok_or(AppError::NotConnected)?;
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Get chunk_id (fast — no BYTEA)
    let chunk_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM image_chunk WHERE parent_page = $1 ORDER BY id ASC LIMIT 1",
    )
    .bind(page_id)
    .fetch_optional(&pool)
    .await?;

    let (chunk_id,) = chunk_id
        .ok_or_else(|| AppError::NotFound(format!("Page {} has no image chunks", page_id)))?;

    // Check cache
    let (thumbnail_path, exists) = check_and_get_thumbnail_path(&cache, &db_name, chunk_id)?;
    if exists {
        return Ok(thumbnail_path.to_string_lossy().to_string());
    }

    // Cache miss — fetch pre-rendered bytes from image_chunk (fast, no re-rendering)
    let row: (Vec<u8>,) = sqlx::query_as(
        "SELECT contents FROM image_chunk WHERE id = $1",
    )
    .bind(chunk_id)
    .fetch_one(&pool)
    .await?;

    generate_thumbnail(&cache, &row.0, &db_name, chunk_id)?;
    Ok(thumbnail_path.to_string_lossy().to_string())
}

/// Get preview URL — cached high-res WebP preview, generated from source file or BYTEA
#[tauri::command]
pub async fn get_preview_url(
    page_id: i64,
    state: State<'_, AppState>,
    cache: State<'_, Mutex<Option<CacheManager>>>,
) -> Result<String> {
    let db_name = state
        .get_db_identifier()
        .await
        .ok_or(AppError::NotConnected)?;
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Get chunk_id (fast — no BYTEA)
    let chunk_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM image_chunk WHERE parent_page = $1 ORDER BY id ASC LIMIT 1",
    )
    .bind(page_id)
    .fetch_optional(&pool)
    .await?;

    let (chunk_id,) = chunk_id
        .ok_or_else(|| AppError::NotFound(format!("Page {} has no image chunks", page_id)))?;

    // Check cache
    let (preview_path, exists) = check_and_get_preview_path(&cache, &db_name, chunk_id)?;
    if exists {
        return Ok(preview_path.to_string_lossy().to_string());
    }

    // Cache miss — fetch pre-rendered bytes from image_chunk (fast, no re-rendering)
    let row: (Vec<u8>,) = sqlx::query_as(
        "SELECT contents FROM image_chunk WHERE id = $1",
    )
    .bind(chunk_id)
    .fetch_one(&pool)
    .await?;

    generate_preview(&cache, &row.0, &db_name, chunk_id)?;
    Ok(preview_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn clear_cache(cache: State<'_, Mutex<Option<CacheManager>>>) -> Result<bool> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    cm.clear_cache()?;
    Ok(true)
}

#[tauri::command]
pub async fn clear_db_cache(
    state: State<'_, AppState>,
    cache: State<'_, Mutex<Option<CacheManager>>>,
) -> Result<bool> {
    let db_name = state
        .get_db_identifier()
        .await
        .ok_or(AppError::NotConnected)?;
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    cm.clear_db_cache(&db_name)?;
    Ok(true)
}

#[tauri::command]
pub async fn get_cache_size(cache: State<'_, Mutex<Option<CacheManager>>>) -> Result<u64> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cm = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    cm.get_cache_size()
}

/// Prefetch thumbnails for all pages in a document.
/// Fetches metadata only (no bulk BYTEA), then resolves bytes per-page from source files.
/// Falls back to BYTEA for pages without available source.
#[tauri::command]
pub async fn prefetch_document_thumbnails(
    document_id: i64,
    state: State<'_, AppState>,
    cache: State<'_, Mutex<Option<CacheManager>>>,
) -> Result<i32> {
    let db_name = state
        .get_db_identifier()
        .await
        .ok_or(AppError::NotConnected)?;
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Stage 1: Get chunk_ids for all pages in the document (metadata only — no BYTEA)
    let rows: Vec<(i64,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (p.id) ic.id
        FROM page p
        JOIN image_chunk ic ON ic.parent_page = p.id
        WHERE p.document_id = $1
        ORDER BY p.id, ic.id ASC
        "#,
    )
    .bind(document_id)
    .fetch_all(&pool)
    .await?;

    // Filter to only uncached chunk_ids
    let uncached: Vec<i64> = rows
        .iter()
        .filter_map(|(chunk_id,)| {
            match has_thumbnail(&cache, &db_name, *chunk_id) {
                Ok(true) => None,
                _ => Some(*chunk_id),
            }
        })
        .collect();

    if uncached.is_empty() {
        return Ok(0);
    }

    // Batch-fetch BYTEA from image_chunk for all uncached chunks in one query
    let chunk_rows: Vec<(i64, Vec<u8>)> = sqlx::query_as(
        "SELECT id, contents FROM image_chunk WHERE id = ANY($1)",
    )
    .bind(&uncached)
    .fetch_all(&pool)
    .await?;

    let mut generated = 0;
    for (chunk_id, contents) in &chunk_rows {
        if generate_thumbnail(&cache, contents, &db_name, *chunk_id).is_ok() {
            generated += 1;
        }
    }

    Ok(generated)
}
