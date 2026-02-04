use base64::{engine::general_purpose::STANDARD, Engine};
use std::sync::Mutex;
use tauri::State;

use crate::cache::CacheManager;
use crate::error::{AppError, Result};
use crate::state::AppState;

/// Get page image as a data URL from bytea column
/// This serves images directly from the database's image_contents column
#[tauri::command]
pub async fn get_page_image_url(page_id: i64, state: State<'_, AppState>) -> Result<String> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let row: (Option<Vec<u8>>, Option<String>) = sqlx::query_as(
        r#"
        SELECT image_contents, mimetype
        FROM page
        WHERE id = $1
        "#,
    )
    .bind(page_id)
    .fetch_one(&pool)
    .await?;

    let (image_contents, mimetype) = row;

    let contents = image_contents
        .ok_or_else(|| AppError::NotFound(format!("Page {} has no image contents", page_id)))?;

    let mime = mimetype.unwrap_or_else(|| "image/png".to_string());
    let base64_data = STANDARD.encode(&contents);
    let data_url = format!("data:{};base64,{}", mime, base64_data);

    Ok(data_url)
}

/// Get image chunk as a data URL from bytea column
#[tauri::command]
pub async fn get_chunk_image_url(chunk_id: i64, state: State<'_, AppState>) -> Result<String> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let row: (Vec<u8>, String) = sqlx::query_as(
        r#"
        SELECT contents, mimetype
        FROM image_chunk
        WHERE id = $1
        "#,
    )
    .bind(chunk_id)
    .fetch_one(&pool)
    .await?;

    let (contents, mimetype) = row;
    let base64_data = STANDARD.encode(&contents);
    let data_url = format!("data:{};base64,{}", mimetype, base64_data);

    Ok(data_url)
}

/// Helper to check if thumbnail exists and get path (without holding lock across await)
fn check_and_get_thumbnail_path(
    cache: &Mutex<Option<CacheManager>>,
    db_name: &str,
    chunk_id: i64,
) -> Result<(std::path::PathBuf, bool)> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;

    let path = cache_manager.thumbnail_path(db_name, &chunk_id);
    let exists = path.exists();
    Ok((path, exists))
}

/// Helper to generate thumbnail from bytes
fn generate_thumbnail(
    cache: &Mutex<Option<CacheManager>>,
    image_bytes: &[u8],
    db_name: &str,
    chunk_id: i64,
) -> Result<std::path::PathBuf> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;

    cache_manager.generate_thumbnail_from_bytes(image_bytes, db_name, &chunk_id)
}

/// Get thumbnail URL - returns cached WebP thumbnail or generates from page's first image chunk
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

    // Stage 1: Get chunk_id only (fast - no bytea transfer)
    let chunk_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM image_chunk WHERE parent_page = $1 ORDER BY id ASC LIMIT 1",
    )
    .bind(page_id)
    .fetch_optional(&pool)
    .await?;

    let (chunk_id,) = chunk_id
        .ok_or_else(|| AppError::NotFound(format!("Page {} has no image chunks", page_id)))?;

    // Check cache BEFORE loading image data
    let (thumbnail_path, exists) = check_and_get_thumbnail_path(&cache, &db_name, chunk_id)?;

    if exists {
        return Ok(thumbnail_path.to_string_lossy().to_string());
    }

    // Stage 2: Cache miss - now fetch the full image
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT contents FROM image_chunk WHERE id = $1")
        .bind(chunk_id)
        .fetch_optional(&pool)
        .await?;

    let (contents,) =
        row.ok_or_else(|| AppError::NotFound(format!("Chunk {} not found", chunk_id)))?;

    // Generate thumbnail with chunk_id as cache key
    generate_thumbnail(&cache, &contents, &db_name, chunk_id)?;
    Ok(thumbnail_path.to_string_lossy().to_string())
}

/// Helper to check if preview exists and get path (without holding lock across await)
fn check_and_get_preview_path(
    cache: &Mutex<Option<CacheManager>>,
    db_name: &str,
    chunk_id: i64,
) -> Result<(std::path::PathBuf, bool)> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;

    let path = cache_manager.preview_path(db_name, &chunk_id);
    let exists = path.exists();
    Ok((path, exists))
}

/// Helper to generate preview from bytes
fn generate_preview(
    cache: &Mutex<Option<CacheManager>>,
    image_bytes: &[u8],
    db_name: &str,
    chunk_id: i64,
) -> Result<std::path::PathBuf> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;

    cache_manager.generate_preview_from_bytes(image_bytes, db_name, &chunk_id)
}

/// Get preview URL - returns cached high-res preview or generates from page's first image chunk
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

    // Stage 1: Get chunk_id only (fast - no bytea transfer)
    let chunk_id: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM image_chunk WHERE parent_page = $1 ORDER BY id ASC LIMIT 1",
    )
    .bind(page_id)
    .fetch_optional(&pool)
    .await?;

    let (chunk_id,) = chunk_id
        .ok_or_else(|| AppError::NotFound(format!("Page {} has no image chunks", page_id)))?;

    // Check cache BEFORE loading image data
    let (preview_path, exists) = check_and_get_preview_path(&cache, &db_name, chunk_id)?;

    if exists {
        return Ok(preview_path.to_string_lossy().to_string());
    }

    // Stage 2: Cache miss - now fetch the full image
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT contents FROM image_chunk WHERE id = $1")
        .bind(chunk_id)
        .fetch_optional(&pool)
        .await?;

    let (contents,) =
        row.ok_or_else(|| AppError::NotFound(format!("Chunk {} not found", chunk_id)))?;

    // Generate preview with chunk_id as cache key
    generate_preview(&cache, &contents, &db_name, chunk_id)?;
    Ok(preview_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn clear_cache(cache: State<'_, Mutex<Option<CacheManager>>>) -> Result<bool> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;

    cache_manager.clear_cache()?;
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
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;

    cache_manager.clear_db_cache(&db_name)?;
    Ok(true)
}

#[tauri::command]
pub async fn get_cache_size(cache: State<'_, Mutex<Option<CacheManager>>>) -> Result<u64> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;

    cache_manager.get_cache_size()
}

/// Helper to check if thumbnail exists for a chunk
fn has_thumbnail(
    cache: &Mutex<Option<CacheManager>>,
    db_name: &str,
    chunk_id: i64,
) -> Result<bool> {
    let cache_guard = cache.lock().map_err(|e| AppError::Cache(e.to_string()))?;
    let cache_manager = cache_guard
        .as_ref()
        .ok_or_else(|| AppError::Cache("Cache manager not initialized".to_string()))?;
    Ok(cache_manager.has_thumbnail(db_name, &chunk_id))
}

/// Prefetch thumbnails for all pages in a document
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

    // Get all pages for document with their first image chunk id and contents
    let pages: Vec<(i64, i64, Vec<u8>)> = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (p.id) p.id, ic.id, ic.contents
        FROM page p
        JOIN image_chunk ic ON ic.parent_page = p.id
        WHERE p.document_id = $1
        ORDER BY p.id, ic.id ASC
        "#,
    )
    .bind(document_id)
    .fetch_all(&pool)
    .await?;

    let mut generated = 0;
    for (_page_id, chunk_id, contents) in pages {
        // Check thumbnail exists using chunk_id (drops lock immediately)
        if !has_thumbnail(&cache, &db_name, chunk_id)? {
            // Generate thumbnail (drops lock after generation)
            if generate_thumbnail(&cache, &contents, &db_name, chunk_id).is_ok() {
                generated += 1;
            }
        }
    }

    Ok(generated)
}
