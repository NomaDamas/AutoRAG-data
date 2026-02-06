use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{AppError, Result};

/// Monotonic counter to ensure unique temp file names across concurrent calls
static RENDER_COUNTER: AtomicU64 = AtomicU64::new(0);

use super::types::PdfMetadata;

/// Process a PDF file using poppler's pdftoppm and pdfinfo commands
pub fn process_pdf(path: &Path) -> Result<PdfProcessingResult> {
    // Get PDF info (page count, metadata)
    let metadata = get_pdf_info(path)?;
    let page_count = metadata.page_count;

    // Render all pages to PNG
    let mut pages = Vec::with_capacity(page_count as usize);
    for page_num in 1..=page_count {
        let png_bytes = render_page_to_png(path, page_num)?;
        pages.push(png_bytes);
    }

    Ok(PdfProcessingResult {
        page_count,
        metadata: PdfMetadata {
            title: metadata.title,
            author: metadata.author,
        },
        pages,
    })
}

/// Result of processing a PDF file
pub struct PdfProcessingResult {
    pub page_count: i32,
    pub metadata: PdfMetadata,
    pub pages: Vec<Vec<u8>>,
}

struct PdfInfo {
    page_count: i32,
    title: Option<String>,
    author: Option<String>,
}

/// Get PDF metadata using pdfinfo command
fn get_pdf_info(path: &Path) -> Result<PdfInfo> {
    let output = Command::new("pdfinfo").arg(path).output().map_err(|e| {
        AppError::PdfError(format!(
            "Failed to run pdfinfo: {}. Is poppler installed?",
            e
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::PdfError(format!("pdfinfo failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut page_count = 0;
    let mut title = None;
    let mut author = None;

    for line in stdout.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Pages" => {
                    page_count = value.parse().unwrap_or(0);
                }
                "Title" => {
                    if !value.is_empty() {
                        title = Some(value.to_string());
                    }
                }
                "Author" => {
                    if !value.is_empty() {
                        author = Some(value.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    if page_count == 0 {
        return Err(AppError::PdfError(
            "Could not determine page count".to_string(),
        ));
    }

    Ok(PdfInfo {
        page_count,
        title,
        author,
    })
}

/// Render a single page to PNG bytes at 150 DPI using pdftoppm
pub fn render_page_to_png(path: &Path, page_num: i32) -> Result<Vec<u8>> {
    // Create a unique temporary file prefix per call to avoid races
    let temp_dir = std::env::temp_dir();
    let counter = RENDER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let output_prefix = temp_dir.join(format!("autorag_page_{}_{}", std::process::id(), counter));

    let output = Command::new("pdftoppm")
        .args([
            "-png", // Output PNG format
            "-r",
            "150", // 150 DPI
            "-f",
            &page_num.to_string(), // First page
            "-l",
            &page_num.to_string(), // Last page (same = single page)
            "-singlefile",         // Don't add page number suffix
        ])
        .arg(path)
        .arg(&output_prefix)
        .output()
        .map_err(|e| {
            AppError::PdfError(format!(
                "Failed to run pdftoppm: {}. Is poppler installed?",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::PdfError(format!(
            "pdftoppm failed on page {}: {}",
            page_num, stderr
        )));
    }

    // Read the output file (pdftoppm adds .png extension)
    let output_file = format!("{}.png", output_prefix.display());
    let png_bytes = fs::read(&output_file).map_err(|e| {
        AppError::PdfError(format!("Failed to read rendered page {}: {}", page_num, e))
    })?;

    // Clean up the temporary file
    let _ = fs::remove_file(&output_file);

    if png_bytes.is_empty() {
        return Err(AppError::PdfError(format!(
            "pdftoppm produced empty output for page {}",
            page_num
        )));
    }

    Ok(png_bytes)
}
