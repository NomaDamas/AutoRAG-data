use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};
use tokio::task::spawn_blocking;

use crate::error::{AppError, Result};
use crate::ingest::{process_pdf, IngestionProgress, IngestionResult};
use crate::state::AppState;

/// Ingest a PDF file into the database
#[tauri::command]
pub async fn ingest_pdf(
    file_path: String,
    title: Option<String>,
    author: Option<String>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<IngestionResult> {
    // Get database pool
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let path = PathBuf::from(&file_path);

    // Extract filename from path
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // Emit reading progress
    let _ = app_handle.emit("ingestion-progress", IngestionProgress::reading(0));

    // Process PDF in a blocking task (PDFium is not Send)
    let pdf_result = spawn_blocking(move || process_pdf(&path))
        .await
        .map_err(|e| AppError::PdfError(format!("Task join error: {}", e)))??;

    let page_count = pdf_result.page_count;

    // Use provided title/author or fall back to PDF metadata
    let final_title = title.or(pdf_result.metadata.title);
    let final_author = author.or(pdf_result.metadata.author);

    // Emit progress with page count
    let _ = app_handle.emit("ingestion-progress", IngestionProgress::reading(page_count));

    // Begin transaction
    let mut tx = pool.begin().await?;

    // Insert file record
    let file_id: i64 =
        sqlx::query_scalar(r#"INSERT INTO file (type, path) VALUES ('raw', $1) RETURNING id"#)
            .bind(&file_path)
            .fetch_one(&mut *tx)
            .await?;

    // Insert document record
    let document_id: i64 = sqlx::query_scalar(
        r#"INSERT INTO document (path, filename, author, title) VALUES ($1, $2, $3, $4) RETURNING id"#,
    )
    .bind(file_id)
    .bind(&filename)
    .bind(&final_author)
    .bind(&final_title)
    .fetch_one(&mut *tx)
    .await?;

    let mut image_chunk_count = 0;
    let mimetype = "image/png".to_string();

    // Insert pages and chunks
    for (page_idx, png_bytes) in pdf_result.pages.into_iter().enumerate() {
        // Emit progress
        let _ = app_handle.emit(
            "ingestion-progress",
            IngestionProgress::rendering((page_idx + 1) as i32, page_count),
        );

        // Insert page record
        let page_id: i64 = sqlx::query_scalar(
            r#"INSERT INTO page (page_num, document_id, image_contents, mimetype)
               VALUES ($1, $2, $3, $4) RETURNING id"#,
        )
        .bind((page_idx + 1) as i32) // 1-indexed page number
        .bind(document_id)
        .bind(&png_bytes)
        .bind(&mimetype)
        .fetch_one(&mut *tx)
        .await?;

        // Insert image_chunk record (same image as page for now)
        sqlx::query(
            r#"INSERT INTO image_chunk (parent_page, contents, mimetype)
               VALUES ($1, $2, $3)"#,
        )
        .bind(page_id)
        .bind(&png_bytes)
        .bind(&mimetype)
        .execute(&mut *tx)
        .await?;

        image_chunk_count += 1;
    }

    // Commit transaction
    tx.commit().await?;

    // Emit complete progress
    let _ = app_handle.emit(
        "ingestion-progress",
        IngestionProgress::complete(page_count),
    );

    Ok(IngestionResult {
        file_id,
        document_id,
        page_count,
        image_chunk_count,
    })
}

/// Get supported file formats for ingestion
#[tauri::command]
pub fn get_supported_formats() -> Vec<&'static str> {
    vec!["pdf"]
}
