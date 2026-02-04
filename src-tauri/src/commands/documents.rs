use tauri::State;

use crate::db::{
    Document, DocumentWithPages, File, FileWithDocuments, ImageChunkInfo, PageInfo, PageWithChunks,
};
use crate::error::{AppError, Result};
use crate::state::AppState;

#[tauri::command]
pub async fn list_files(state: State<'_, AppState>) -> Result<Vec<File>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let files = sqlx::query_as::<_, File>(
        r#"
        SELECT id, "type", path
        FROM file
        ORDER BY path ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(files)
}

#[tauri::command]
pub async fn list_files_with_documents(
    state: State<'_, AppState>,
) -> Result<Vec<FileWithDocuments>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let files = sqlx::query_as::<_, File>(
        r#"
        SELECT id, "type", path
        FROM file
        ORDER BY path ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let mut result = Vec::new();
    for file in files {
        let documents = sqlx::query_as::<_, Document>(
            r#"
            SELECT id, path, filename, author, title, doc_metadata
            FROM document
            WHERE path = $1
            ORDER BY title ASC NULLS LAST
            "#,
        )
        .bind(file.id)
        .fetch_all(&pool)
        .await?;

        result.push(FileWithDocuments { file, documents });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_document(document_id: i64, state: State<'_, AppState>) -> Result<Document> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let document = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, path, filename, author, title, doc_metadata
        FROM document
        WHERE id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Document {} not found", document_id)))?;

    Ok(document)
}

#[tauri::command]
pub async fn get_document_with_pages(
    document_id: i64,
    state: State<'_, AppState>,
) -> Result<DocumentWithPages> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let document = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, path, filename, author, title, doc_metadata
        FROM document
        WHERE id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Document {} not found", document_id)))?;

    // Fetch pages without image_contents (too large for list response)
    let pages = sqlx::query_as::<_, PageInfo>(
        r#"
        SELECT id, page_num, document_id, mimetype, page_metadata
        FROM page
        WHERE document_id = $1
        ORDER BY page_num ASC
        "#,
    )
    .bind(document_id)
    .fetch_all(&pool)
    .await?;

    let mut pages_with_chunks = Vec::new();
    for page in pages {
        // Fetch chunks without binary contents
        let chunks = sqlx::query_as::<_, ImageChunkInfo>(
            r#"
            SELECT id, parent_page, mimetype
            FROM image_chunk
            WHERE parent_page = $1
            ORDER BY id ASC
            "#,
        )
        .bind(page.id)
        .fetch_all(&pool)
        .await?;

        pages_with_chunks.push(PageWithChunks { page, chunks });
    }

    Ok(DocumentWithPages {
        document,
        pages: pages_with_chunks,
    })
}

#[tauri::command]
pub async fn get_pages(document_id: i64, state: State<'_, AppState>) -> Result<Vec<PageInfo>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let pages = sqlx::query_as::<_, PageInfo>(
        r#"
        SELECT id, page_num, document_id, mimetype, page_metadata
        FROM page
        WHERE document_id = $1
        ORDER BY page_num ASC
        "#,
    )
    .bind(document_id)
    .fetch_all(&pool)
    .await?;

    Ok(pages)
}

#[tauri::command]
pub async fn get_page_chunks(
    page_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ImageChunkInfo>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let chunks = sqlx::query_as::<_, ImageChunkInfo>(
        r#"
        SELECT id, parent_page, mimetype
        FROM image_chunk
        WHERE parent_page = $1
        ORDER BY id ASC
        "#,
    )
    .bind(page_id)
    .fetch_all(&pool)
    .await?;

    Ok(chunks)
}

#[tauri::command]
pub async fn get_file_path(document_id: i64, state: State<'_, AppState>) -> Result<String> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let path: (String,) = sqlx::query_as(
        r#"
        SELECT f.path
        FROM file f
        JOIN document d ON d.path = f.id
        WHERE d.id = $1
        "#,
    )
    .bind(document_id)
    .fetch_one(&pool)
    .await?;

    Ok(path.0)
}

/// Get page count for a document
#[tauri::command]
pub async fn get_document_page_count(document_id: i64, state: State<'_, AppState>) -> Result<i64> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM page
        WHERE document_id = $1
        "#,
    )
    .bind(document_id)
    .fetch_one(&pool)
    .await?;

    Ok(count.0)
}

/// List all documents (without file grouping)
#[tauri::command]
pub async fn list_documents(state: State<'_, AppState>) -> Result<Vec<Document>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let documents = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, path, filename, author, title, doc_metadata
        FROM document
        ORDER BY title ASC NULLS LAST
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(documents)
}
