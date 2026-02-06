use serde::Serialize;
use tauri::State;

use crate::error::{AppError, Result};
use crate::state::AppState;

/// Get the source file path for a document (from the `file` table).
/// Returns None if the document has no linked file record.
#[tauri::command]
pub async fn get_source_file_url(
    document_id: i64,
    state: State<'_, AppState>,
) -> Result<Option<String>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT f.path
        FROM file f
        JOIN document d ON d.path = f.id
        WHERE d.id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(&pool)
    .await?;

    Ok(row.map(|(path,)| path))
}

#[derive(Serialize)]
pub struct PageSourceInfo {
    pub page_id: i64,
    pub chunk_ids: Vec<i64>,
    pub page_num: i32,
    pub source_path: Option<String>,
}

/// Get per-page source paths for all pages in a document.
/// For image documents, source_path comes from page_metadata.
/// For PDF documents, source_path comes from the file table.
#[tauri::command]
pub async fn get_page_source_urls(
    document_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<PageSourceInfo>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let rows: Vec<(i64, i32, Option<String>)> = sqlx::query_as(
        r#"
        SELECT p.id, p.page_num,
               COALESCE(p.page_metadata->>'source_path', f.path) as source_path
        FROM page p
        JOIN document d ON p.document_id = d.id
        LEFT JOIN file f ON d.path = f.id
        WHERE p.document_id = $1
        ORDER BY p.page_num
        "#,
    )
    .bind(document_id)
    .fetch_all(&pool)
    .await?;

    let page_ids: Vec<i64> = rows.iter().map(|(id, _, _)| *id).collect();

    // Batch-fetch chunk IDs for all pages
    let chunk_rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT parent_page, id FROM image_chunk WHERE parent_page = ANY($1) ORDER BY id",
    )
    .bind(&page_ids)
    .fetch_all(&pool)
    .await?;

    let mut result: Vec<PageSourceInfo> = rows
        .into_iter()
        .map(|(page_id, page_num, source_path)| PageSourceInfo {
            page_id,
            chunk_ids: Vec::new(),
            page_num,
            source_path,
        })
        .collect();

    // Assign chunk IDs to their pages
    for (parent_page, chunk_id) in chunk_rows {
        if let Some(info) = result.iter_mut().find(|p| p.page_id == parent_page) {
            info.chunk_ids.push(chunk_id);
        }
    }

    Ok(result)
}

/// Get a chunk's image data as a data: URL (BYTEA fallback).
/// Always works regardless of whether the original file is accessible.
#[tauri::command]
pub async fn get_chunk_data_url(
    chunk_id: i64,
    state: State<'_, AppState>,
) -> Result<String> {
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
