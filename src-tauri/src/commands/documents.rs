use tauri::State;

use crate::db::{
    Document, DocumentDeletionCheck, DocumentWithPages, File, FileWithDocuments, ImageChunkInfo,
    PageInfo, PageWithChunks, Query,
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

/// Check whether a document can be safely deleted by looking for
/// queries that reference its chunks as retrieval ground truth evidence.
#[tauri::command]
pub async fn check_document_deletable(
    document_id: i64,
    state: State<'_, AppState>,
) -> Result<DocumentDeletionCheck> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let blocking_queries = sqlx::query_as::<_, Query>(
        r#"
        SELECT DISTINCT q.id, q.contents, q.query_to_llm, q.generation_gt
        FROM query q
        JOIN retrieval_relation rr ON rr.query_id = q.id
        JOIN image_chunk ic ON ic.id = rr.image_chunk_id
        JOIN page p ON p.id = ic.parent_page
        WHERE p.document_id = $1
        ORDER BY q.id
        "#,
    )
    .bind(document_id)
    .fetch_all(&pool)
    .await?;

    Ok(DocumentDeletionCheck {
        deletable: blocking_queries.is_empty(),
        blocking_queries,
    })
}

/// Delete a document and all its dependent data (pages, chunks, etc.)
/// Fails if any queries reference this document's chunks as evidence.
#[tauri::command]
pub async fn delete_document(document_id: i64, state: State<'_, AppState>) -> Result<bool> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Safety re-check: ensure no retrieval relations reference this document's chunks
    let blocking_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM retrieval_relation rr
        JOIN image_chunk ic ON ic.id = rr.image_chunk_id
        JOIN page p ON p.id = ic.parent_page
        WHERE p.document_id = $1
        "#,
    )
    .bind(document_id)
    .fetch_one(&pool)
    .await?;

    if blocking_count.0 > 0 {
        return Err(AppError::Custom(format!(
            "Cannot delete document {}: {} retrieval relation(s) still reference its chunks",
            document_id, blocking_count.0
        )));
    }

    // Fetch document to get file FK
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

    // Collect page IDs for this document
    let page_ids: Vec<i64> = sqlx::query_scalar(
        r#"SELECT id FROM page WHERE document_id = $1"#,
    )
    .bind(document_id)
    .fetch_all(&pool)
    .await?;

    // Begin transaction — delete in FK-safe order
    let mut tx = pool.begin().await?;

    if !page_ids.is_empty() {
        // Delete image_chunk_retrieved_result for chunks belonging to this document's pages
        sqlx::query(
            r#"
            DELETE FROM image_chunk_retrieved_result
            WHERE image_chunk_id IN (
                SELECT id FROM image_chunk WHERE parent_page = ANY($1)
            )
            "#,
        )
        .bind(&page_ids)
        .execute(&mut *tx)
        .await?;

        // Delete image_chunk rows
        sqlx::query(r#"DELETE FROM image_chunk WHERE parent_page = ANY($1)"#)
            .bind(&page_ids)
            .execute(&mut *tx)
            .await?;

        // Delete page_chunk_relation rows
        sqlx::query(r#"DELETE FROM page_chunk_relation WHERE page_id = ANY($1)"#)
            .bind(&page_ids)
            .execute(&mut *tx)
            .await?;
    }

    // Delete pages
    sqlx::query(r#"DELETE FROM page WHERE document_id = $1"#)
        .bind(document_id)
        .execute(&mut *tx)
        .await?;

    // Delete document
    sqlx::query(r#"DELETE FROM document WHERE id = $1"#)
        .bind(document_id)
        .execute(&mut *tx)
        .await?;

    // Clean up orphaned file record if no other documents reference it
    if let Some(file_id) = document.path {
        let other_docs: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM document WHERE path = $1"#,
        )
        .bind(file_id)
        .fetch_one(&mut *tx)
        .await?;

        if other_docs.0 == 0 {
            sqlx::query(r#"DELETE FROM file WHERE id = $1"#)
                .bind(file_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;

    Ok(true)
}
