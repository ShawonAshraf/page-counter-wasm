//! Data structures and types for page count estimation.
//!
//! This module defines the core types used throughout the page counter library,
//! including error types, configuration options, and result structures.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during page count estimation.
///
/// This enum represents all possible error conditions that may arise when
/// processing documents of various formats (PDF, XLSX, DOCX, etc.).
#[derive(Debug, Error)]
pub enum EstimatorError {
    /// The file format is not supported or could not be recognized.
    #[error("Unsupported or unrecognized format")]
    UnsupportedFormat,
    /// An error occurred while parsing a PDF document.
    #[error("PDF parse error: {0}")]
    PdfError(String),
    /// An error occurred while parsing an Excel (XLSX) document.
    #[error("XLSX parse error: {0}")]
    XlsxError(String),
    /// A general error occurred during processing.
    #[error("General error: {0}")]
    General(String),
}

/// Represents the physical dimensions of a page in millimeters.
///
/// This structure is used to describe the size of individual pages in documents,
/// particularly useful for PDFs where each page can have different dimensions.
#[derive(Serialize, Deserialize, Clone)]
pub struct PageSizeMm {
    /// The width of the page in millimeters.
    pub width_mm: f64,
    /// The height of the page in millimeters.
    pub height_mm: f64,
}

/// The result of a page count estimation operation.
///
/// Contains the estimated page count, page dimensions, and any relevant notes
/// about how the estimation was performed. This is the primary output structure
/// returned to callers.
#[derive(Serialize, Deserialize)]
pub struct EstimateResult {
    /// Estimated total page count for the document.
    pub page_count: usize,
    /// Per-page sizes when known (e.g., from PDF metadata).
    /// For non-PDF documents, this contains the inferred default size repeated for each page.
    pub page_sizes: Vec<PageSizeMm>,
    /// Textual explanations and notes about the estimation process.
    /// May include information about the method used, assumptions made, or warnings.
    pub notes: Vec<String>,
}

/// Configuration options for customizing page count estimation behavior.
///
/// All fields are optional. When not provided, sensible defaults are used.
/// This structure can be serialized from JSON to allow easy configuration
/// from JavaScript or other calling environments.
///
/// # Examples
///
/// Using default A4 paper size:
/// ```json
/// {}
/// ```
///
/// Specifying Letter paper:
/// ```json
/// { "default_paper": "Letter" }
/// ```
///
/// Using custom paper dimensions:
/// ```json
/// { "custom_paper_mm": [210.0, 297.0] }
/// ```
#[derive(Serialize, Deserialize)]
pub struct EstimateOptions {
    /// Default page size to assume for non-PDF documents.
    /// Supported values: "A4" (210×297mm) or "Letter" (215.9×279.4mm).
    /// Defaults to "A4" if not specified.
    pub default_paper: Option<String>,
    /// Custom paper size in millimeters as a tuple (width, height).
    /// When provided, this takes precedence over `default_paper`.
    pub custom_paper_mm: Option<(f64, f64)>,
    /// Characters per page heuristic for text-based documents.
    /// Overrides the default heuristic when provided.
    /// Useful for documents with known formatting or character density.
    pub chars_per_page: Option<usize>,
    /// Rows per page for spreadsheet documents.
    /// Used to estimate how many pages a spreadsheet would occupy when printed.
    pub rows_per_page: Option<usize>,
}

impl Default for EstimateOptions {
    fn default() -> Self {
        Self {
            default_paper: Some("A4".into()),
            custom_paper_mm: None,
            chars_per_page: None,
            rows_per_page: None,
        }
    }
}
