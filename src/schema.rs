use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EstimatorError {
    #[error("Unsupported or unrecognized format")]
    UnsupportedFormat,
    #[error("PDF parse error: {0}")]
    PdfError(String),
    #[error("XLSX parse error: {0}")]
    XlsxError(String),
    #[error("General error: {0}")]
    General(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PageSizeMm {
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Serialize, Deserialize)]
pub struct EstimateResult {
    /// estimated total page count
    pub page_count: usize,
    /// per-page sizes when known (PDF), otherwise inferred default size repeated
    pub page_sizes: Vec<crate::PageSizeMm>,
    /// textual explanation / notes
    pub notes: Vec<String>,
}

/// Options the user can pass (serialized JSON). All fields optional.
#[derive(Serialize, Deserialize)]
pub struct EstimateOptions {
    /// default page size to assume for non-PDF documents: "A4" or "Letter" or custom mm [w,h]
    pub default_paper: Option<String>,
    /// custom paper size in mm [w,h]; takes precedence over default_paper when provided.
    pub custom_paper_mm: Option<(f64, f64)>,
    /// chars per page heuristic (overrides default heuristic)
    pub chars_per_page: Option<usize>,
    /// rows per page for spreadsheets
    pub rows_per_page: Option<usize>,
}