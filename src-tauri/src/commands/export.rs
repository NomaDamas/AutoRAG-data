use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::error::{AppError, Result};
use crate::state::AppState;

/// Configuration for export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub output_path: String,
    pub create_zip: bool,
    pub include_documents: bool,
    pub include_queries: bool,
    pub include_relations: bool,
    pub include_image_chunks: bool,
    pub include_images: bool,
}

/// Progress update during export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProgress {
    pub phase: String, // "Documents", "Queries", "Relations", "ImageChunks", "Images", "Zipping", "Complete", "Failed"
    pub current: u32,
    pub total: u32,
    pub message: String,
}

impl ExportProgress {
    pub fn documents(current: u32, total: u32) -> Self {
        Self {
            phase: "Documents".to_string(),
            current,
            total,
            message: format!("Exporting documents ({}/{})", current, total),
        }
    }

    pub fn queries(current: u32, total: u32) -> Self {
        Self {
            phase: "Queries".to_string(),
            current,
            total,
            message: format!("Exporting queries ({}/{})", current, total),
        }
    }

    pub fn relations(current: u32, total: u32) -> Self {
        Self {
            phase: "Relations".to_string(),
            current,
            total,
            message: format!("Exporting relations ({}/{})", current, total),
        }
    }

    pub fn image_chunks(current: u32, total: u32) -> Self {
        Self {
            phase: "ImageChunks".to_string(),
            current,
            total,
            message: format!("Exporting image chunks ({}/{})", current, total),
        }
    }

    pub fn images(current: u32, total: u32) -> Self {
        Self {
            phase: "Images".to_string(),
            current,
            total,
            message: format!("Exporting images ({}/{})", current, total),
        }
    }

    pub fn zipping() -> Self {
        Self {
            phase: "Zipping".to_string(),
            current: 0,
            total: 0,
            message: "Creating ZIP archive...".to_string(),
        }
    }

    pub fn complete() -> Self {
        Self {
            phase: "Complete".to_string(),
            current: 0,
            total: 0,
            message: "Export completed successfully".to_string(),
        }
    }

    pub fn failed(message: String) -> Self {
        Self {
            phase: "Failed".to_string(),
            current: 0,
            total: 0,
            message,
        }
    }
}

/// Result of a successful export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub output_path: String,
    pub documents_count: u32,
    pub queries_count: u32,
    pub relations_count: u32,
    pub image_chunks_count: u32,
    pub images_count: u32,
}

/// Counts for export preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportCounts {
    pub documents: u32,
    pub queries: u32,
    pub relations: u32,
    pub image_chunks: u32,
}

/// Get counts for export preview
#[tauri::command]
pub async fn get_export_counts(state: State<'_, AppState>) -> Result<ExportCounts> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let documents: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM document")
        .fetch_one(&pool)
        .await?;

    let queries: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM query")
        .fetch_one(&pool)
        .await?;

    let relations: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM retrieval_relation")
        .fetch_one(&pool)
        .await?;

    let image_chunks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM image_chunk")
        .fetch_one(&pool)
        .await?;

    Ok(ExportCounts {
        documents: documents.0 as u32,
        queries: queries.0 as u32,
        relations: relations.0 as u32,
        image_chunks: image_chunks.0 as u32,
    })
}

/// Export data to CSV files and images
#[tauri::command]
pub async fn export_data(
    config: ExportConfig,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportResult> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Create output directory
    let output_dir = PathBuf::from(&config.output_path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    let mut result = ExportResult {
        output_path: config.output_path.clone(),
        documents_count: 0,
        queries_count: 0,
        relations_count: 0,
        image_chunks_count: 0,
        images_count: 0,
    };

    // Export documents
    if config.include_documents {
        result.documents_count = export_documents_csv(&pool, &output_dir, &app_handle).await?;
    }

    // Export queries
    if config.include_queries {
        result.queries_count = export_queries_csv(&pool, &output_dir, &app_handle).await?;
    }

    // Export relations
    if config.include_relations {
        result.relations_count = export_relations_csv(&pool, &output_dir, &app_handle).await?;
    }

    // Export image chunks metadata
    if config.include_image_chunks {
        result.image_chunks_count =
            export_image_chunks_csv(&pool, &output_dir, &app_handle).await?;
    }

    // Export images
    if config.include_images {
        result.images_count = export_images(&pool, &output_dir, &app_handle).await?;
    }

    // Create ZIP if requested
    if config.create_zip {
        let _ = app_handle.emit("export-progress", ExportProgress::zipping());

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let zip_filename = format!("autorag_export_{}.zip", timestamp);
        let zip_path = output_dir.parent().unwrap_or(&output_dir).join(&zip_filename);

        create_zip_archive(&output_dir, &zip_path)?;

        // Remove the original directory after zipping
        fs::remove_dir_all(&output_dir)?;

        result.output_path = zip_path.to_string_lossy().to_string();
    }

    let _ = app_handle.emit("export-progress", ExportProgress::complete());

    Ok(result)
}

/// Row type for document export
#[derive(sqlx::FromRow)]
struct DocumentRow {
    id: i64,
    filename: Option<String>,
    author: Option<String>,
    title: Option<String>,
    doc_metadata: Option<serde_json::Value>,
}

async fn export_documents_csv(
    pool: &sqlx::PgPool,
    output_dir: &Path,
    app_handle: &AppHandle,
) -> Result<u32> {
    let rows = sqlx::query_as::<_, DocumentRow>(
        "SELECT id, filename, author, title, doc_metadata FROM document ORDER BY id",
    )
    .fetch_all(pool)
    .await?;

    let total = rows.len() as u32;
    let _ = app_handle.emit("export-progress", ExportProgress::documents(0, total));

    let csv_path = output_dir.join("documents.csv");
    let file = File::create(&csv_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    // Write header
    wtr.write_record(["id", "filename", "author", "title", "doc_metadata"])?;

    for (i, row) in rows.iter().enumerate() {
        let metadata_str = row
            .doc_metadata
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_default();

        wtr.write_record([
            row.id.to_string(),
            row.filename.clone().unwrap_or_default(),
            row.author.clone().unwrap_or_default(),
            row.title.clone().unwrap_or_default(),
            metadata_str,
        ])?;

        if (i + 1) % 100 == 0 || i + 1 == rows.len() {
            let _ = app_handle.emit(
                "export-progress",
                ExportProgress::documents((i + 1) as u32, total),
            );
        }
    }

    wtr.flush()?;
    Ok(total)
}

/// Row type for query export
#[derive(sqlx::FromRow)]
struct QueryRow {
    id: i64,
    contents: String,
    query_to_llm: Option<String>,
    generation_gt: Option<Vec<String>>,
}

async fn export_queries_csv(
    pool: &sqlx::PgPool,
    output_dir: &Path,
    app_handle: &AppHandle,
) -> Result<u32> {
    let rows = sqlx::query_as::<_, QueryRow>(
        "SELECT id, contents, query_to_llm, generation_gt FROM query ORDER BY id",
    )
    .fetch_all(pool)
    .await?;

    let total = rows.len() as u32;
    let _ = app_handle.emit("export-progress", ExportProgress::queries(0, total));

    let csv_path = output_dir.join("queries.csv");
    let file = File::create(&csv_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    // Write header
    wtr.write_record(["id", "contents", "query_to_llm", "generation_gt"])?;

    for (i, row) in rows.iter().enumerate() {
        // Join generation_gt with pipe delimiter
        let generation_gt_str = row
            .generation_gt
            .as_ref()
            .map(|v| v.join("|"))
            .unwrap_or_default();

        wtr.write_record([
            row.id.to_string(),
            row.contents.clone(),
            row.query_to_llm.clone().unwrap_or_default(),
            generation_gt_str,
        ])?;

        if (i + 1) % 100 == 0 || i + 1 == rows.len() {
            let _ = app_handle.emit(
                "export-progress",
                ExportProgress::queries((i + 1) as u32, total),
            );
        }
    }

    wtr.flush()?;
    Ok(total)
}

/// Row type for retrieval relation export
#[derive(sqlx::FromRow)]
struct RelationRow {
    query_id: i64,
    group_index: i32,
    group_order: i32,
    chunk_id: Option<i64>,
    image_chunk_id: Option<i64>,
    score: i32,
}

async fn export_relations_csv(
    pool: &sqlx::PgPool,
    output_dir: &Path,
    app_handle: &AppHandle,
) -> Result<u32> {
    let rows = sqlx::query_as::<_, RelationRow>(
        "SELECT query_id, group_index, group_order, chunk_id, image_chunk_id, score
         FROM retrieval_relation
         ORDER BY query_id, group_index, group_order",
    )
    .fetch_all(pool)
    .await?;

    let total = rows.len() as u32;
    let _ = app_handle.emit("export-progress", ExportProgress::relations(0, total));

    let csv_path = output_dir.join("retrieval_relations.csv");
    let file = File::create(&csv_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    // Write header
    wtr.write_record([
        "query_id",
        "group_index",
        "group_order",
        "chunk_id",
        "image_chunk_id",
        "score",
    ])?;

    for (i, row) in rows.iter().enumerate() {
        wtr.write_record([
            row.query_id.to_string(),
            row.group_index.to_string(),
            row.group_order.to_string(),
            row.chunk_id.map(|id| id.to_string()).unwrap_or_default(),
            row.image_chunk_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            row.score.to_string(),
        ])?;

        if (i + 1) % 100 == 0 || i + 1 == rows.len() {
            let _ = app_handle.emit(
                "export-progress",
                ExportProgress::relations((i + 1) as u32, total),
            );
        }
    }

    wtr.flush()?;
    Ok(total)
}

/// Row type for image chunk export (with page info)
#[derive(sqlx::FromRow)]
struct ImageChunkRow {
    id: i64,
    parent_page: Option<i64>,
    mimetype: String,
    page_num: Option<i32>,
    document_id: Option<i64>,
}

async fn export_image_chunks_csv(
    pool: &sqlx::PgPool,
    output_dir: &Path,
    app_handle: &AppHandle,
) -> Result<u32> {
    let rows = sqlx::query_as::<_, ImageChunkRow>(
        "SELECT ic.id, ic.parent_page, ic.mimetype, p.page_num, p.document_id
         FROM image_chunk ic
         LEFT JOIN page p ON ic.parent_page = p.id
         ORDER BY ic.id",
    )
    .fetch_all(pool)
    .await?;

    let total = rows.len() as u32;
    let _ = app_handle.emit("export-progress", ExportProgress::image_chunks(0, total));

    let csv_path = output_dir.join("image_chunks.csv");
    let file = File::create(&csv_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    // Write header
    wtr.write_record(["id", "parent_page", "mimetype", "page_num", "document_id"])?;

    for (i, row) in rows.iter().enumerate() {
        wtr.write_record([
            row.id.to_string(),
            row.parent_page
                .map(|id| id.to_string())
                .unwrap_or_default(),
            row.mimetype.clone(),
            row.page_num.map(|n| n.to_string()).unwrap_or_default(),
            row.document_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
        ])?;

        if (i + 1) % 100 == 0 || i + 1 == rows.len() {
            let _ = app_handle.emit(
                "export-progress",
                ExportProgress::image_chunks((i + 1) as u32, total),
            );
        }
    }

    wtr.flush()?;
    Ok(total)
}

/// Row type for image content export
#[derive(sqlx::FromRow)]
struct ImageContentRow {
    id: i64,
    contents: Vec<u8>,
}

async fn export_images(
    pool: &sqlx::PgPool,
    output_dir: &Path,
    app_handle: &AppHandle,
) -> Result<u32> {
    // Get total count first
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM image_chunk")
        .fetch_one(pool)
        .await?;
    let total = count.0 as u32;

    let _ = app_handle.emit("export-progress", ExportProgress::images(0, total));

    // Create images directory
    let images_dir = output_dir.join("images");
    fs::create_dir_all(&images_dir)?;

    // Stream images one at a time to avoid memory issues
    let mut offset: i64 = 0;
    let batch_size: i64 = 100;
    let mut exported: u32 = 0;

    loop {
        let rows = sqlx::query_as::<_, ImageContentRow>(
            "SELECT id, contents FROM image_chunk ORDER BY id LIMIT $1 OFFSET $2",
        )
        .bind(batch_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        if rows.is_empty() {
            break;
        }

        for row in &rows {
            let image_path = images_dir.join(format!("{}.png", row.id));
            let mut file = File::create(&image_path)?;
            file.write_all(&row.contents)?;
            exported += 1;

            if exported % 10 == 0 || exported == total {
                let _ = app_handle.emit("export-progress", ExportProgress::images(exported, total));
            }
        }

        offset += batch_size;
    }

    Ok(exported)
}

fn create_zip_archive(source_dir: &Path, zip_path: &Path) -> Result<()> {
    let file = File::create(zip_path)?;
    let writer = BufWriter::new(file);
    let mut zip = ZipWriter::new(writer);

    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Walk through the source directory
    add_dir_to_zip(&mut zip, source_dir, source_dir, options)?;

    zip.finish()?;
    Ok(())
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    base_dir: &Path,
    current_dir: &Path,
    options: FileOptions<()>,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(base_dir).unwrap_or(&path);
        let name = relative_path.to_string_lossy();

        if path.is_dir() {
            zip.add_directory(&format!("{}/", name), options)?;
            add_dir_to_zip(zip, base_dir, &path, options)?;
        } else {
            zip.start_file(name.to_string(), options)?;
            let content = fs::read(&path)?;
            zip.write_all(&content)?;
        }
    }
    Ok(())
}
