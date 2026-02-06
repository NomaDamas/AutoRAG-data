use serde::{Deserialize, Serialize};

/// Progress update during PDF ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionProgress {
    pub current_page: i32,
    pub total_pages: i32,
    pub phase: String, // "Reading", "Rendering", "Complete", "Failed"
    pub message: String,
}

impl IngestionProgress {
    pub fn reading(total_pages: i32) -> Self {
        Self {
            current_page: 0,
            total_pages,
            phase: "Reading".to_string(),
            message: format!("Reading PDF ({} pages)", total_pages),
        }
    }

    pub fn rendering(current_page: i32, total_pages: i32) -> Self {
        Self {
            current_page,
            total_pages,
            phase: "Rendering".to_string(),
            message: format!("Rendering page {} of {}", current_page, total_pages),
        }
    }

    pub fn caching(current_page: i32, total_pages: i32) -> Self {
        Self {
            current_page,
            total_pages,
            phase: "Caching".to_string(),
            message: format!("Caching page {} of {}", current_page, total_pages),
        }
    }

    pub fn complete(total_pages: i32) -> Self {
        Self {
            current_page: total_pages,
            total_pages,
            phase: "Complete".to_string(),
            message: format!("Successfully imported {} pages", total_pages),
        }
    }

    pub fn failed(message: String) -> Self {
        Self {
            current_page: 0,
            total_pages: 0,
            phase: "Failed".to_string(),
            message,
        }
    }
}

/// Result of a successful PDF ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionResult {
    pub file_id: i64,
    pub document_id: i64,
    pub page_count: i32,
    pub image_chunk_count: i32,
}

/// Metadata extracted from PDF
#[derive(Debug, Clone, Default)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
}
