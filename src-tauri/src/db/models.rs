use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// File table - stores raw files (PDFs, images, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: i64,        // bigserial
    pub r#type: String, // varchar(255) NOT NULL - "raw", "image", "audio", "video"
    pub path: String,   // varchar(255) NOT NULL
}

/// Document table - parsed documents from files
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: i64,                                 // bigserial
    pub path: Option<i64>,                       // FK to File.id (nullable)
    pub filename: Option<String>,                // varchar(255)
    pub author: Option<String>,                  // varchar(255)
    pub title: Option<String>,                   // varchar(255)
    pub doc_metadata: Option<serde_json::Value>, // jsonb
}

/// Page table - individual pages from documents with image content
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Page {
    pub id: i64,          // bigserial
    pub page_num: i32,    // NOT NULL
    pub document_id: i64, // FK to Document.id NOT NULL
    #[sqlx(default)]
    #[serde(skip_serializing)]
    pub image_contents: Option<Vec<u8>>, // bytea (page image) - skip in JSON response
    pub mimetype: Option<String>, // varchar(255)
    pub page_metadata: Option<serde_json::Value>, // jsonb
}

/// Page without image contents for list responses
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PageInfo {
    pub id: i64,
    pub page_num: i32,
    pub document_id: i64,
    pub mimetype: Option<String>,
    pub page_metadata: Option<serde_json::Value>,
}

/// ImageChunk table - cropped image regions from pages
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ImageChunk {
    pub id: i64,                  // bigserial
    pub parent_page: Option<i64>, // FK to Page.id
    #[sqlx(default)]
    #[serde(skip_serializing)]
    pub contents: Vec<u8>, // bytea NOT NULL (cropped image) - skip in JSON
    pub mimetype: String,         // varchar(255) NOT NULL
}

/// ImageChunk without binary contents for list responses
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ImageChunkInfo {
    pub id: i64,
    pub parent_page: Option<i64>,
    pub mimetype: String,
}

/// Query table - user questions for RAG benchmarks
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Query {
    pub id: i64,                            // bigserial
    pub contents: String,                   // text NOT NULL
    pub query_to_llm: Option<String>,       // text
    pub generation_gt: Option<Vec<String>>, // text[] - multiple valid answers
}

/// RetrievalRelation table - links queries to evidence chunks
/// Composite PK: (query_id, group_index, group_order)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RetrievalRelation {
    pub query_id: i64,         // FK to Query.id NOT NULL
    pub group_index: i32,      // NOT NULL - which answer group
    pub group_order: i32,      // NOT NULL - rank within group
    pub chunk_id: Option<i64>, // FK to Chunk.id (text evidence)
    pub image_chunk_id: Option<i64>, // FK to ImageChunk.id (image evidence)
    pub score: i32, // Relevance score: 0=not relevant, 1=somewhat relevant (default), 2=highly relevant
                               // Constraint: exactly one of chunk_id or image_chunk_id must be non-null
}

// ============================================================================
// Composite types for API responses
// ============================================================================

/// File with its associated documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWithDocuments {
    pub file: File,
    pub documents: Vec<Document>,
}

/// Document with its pages (without binary image data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentWithPages {
    pub document: Document,
    pub pages: Vec<PageWithChunks>,
}

/// Page with its image chunks (without binary data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageWithChunks {
    pub page: PageInfo,
    pub chunks: Vec<ImageChunkInfo>,
}

/// Query with its associated evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryWithEvidence {
    pub query: Query,
    pub evidence_groups: Vec<EvidenceGroup>,
}

/// A group of evidence items for one valid answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGroup {
    pub group_index: i32,
    pub items: Vec<EvidenceItem>,
}

/// Single evidence item with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub relation: RetrievalRelation,
    pub chunk: Option<ImageChunkInfo>,
    pub page: Option<PageInfo>,
}

// ============================================================================
// Request types for mutations
// ============================================================================

/// Evidence item with chunk_id and score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceWithScore {
    pub chunk_id: i64,
    pub score: i32, // 0=not relevant, 1=somewhat relevant (default), 2=highly relevant
}

/// Request to create a new query with evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQueryRequest {
    pub contents: String,
    pub query_to_llm: Option<String>,
    pub generation_gt: Option<Vec<String>>,
    /// Evidence organized by groups - each inner Vec is a group with ordered chunks and scores
    pub evidence_groups: Vec<Vec<EvidenceWithScore>>,
}

/// Request to update an existing query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQueryRequest {
    pub id: i64,
    pub contents: Option<String>,
    pub query_to_llm: Option<String>,
    pub generation_gt: Option<Vec<String>>,
}

/// Request to add evidence to a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEvidenceRequest {
    pub query_id: i64,
    pub group_index: i32,
    pub image_chunk_id: i64,
    pub score: Option<i32>, // Default to 1 if not provided
}

/// Request to update the score of an existing retrieval relation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScoreRequest {
    pub query_id: i64,
    pub group_index: i32,
    pub group_order: i32,
    pub score: i32,
}
