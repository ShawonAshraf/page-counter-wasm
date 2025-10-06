//! # Assembly Module
//!
//! This module provides the main WASM-exported functions for estimating document page counts.
//! It serves as the bridge between JavaScript and the Rust page estimation logic.
//!
//! ## Overview
//!
//! The module exposes two primary functions:
//! - `estimate_document_base64`: Accepts base64-encoded document data
//! - `estimate_document`: Accepts raw byte arrays
//!
//! Both functions automatically detect the document type (PDF, XLSX, TXT, Markdown) and
//! apply the appropriate estimation algorithm.
//!
//! ## Supported Formats
//!
//! - **PDF**: Uses PDF structure analysis to count pages
//! - **XLSX**: Counts worksheets in Excel files
//! - **TXT**: Estimates pages based on character count and formatting
//! - **Markdown**: Estimates pages considering markdown formatting

use crate::estimators::{
    estimate_markdown_pages, estimate_pdf_pages, estimate_text_pages, estimate_xlsx_pages,
};
use crate::file_utils::detect_type;
use crate::schema::EstimateOptions;
use base64::Engine;
use serde_json::json;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// Estimates the number of pages in a document from base64-encoded data.
///
/// This is a convenience wrapper around `estimate_document` that accepts base64-encoded
/// document bytes. This is particularly useful when working with JavaScript environments
/// where handling raw byte arrays can be cumbersome.
///
/// # Parameters
///
/// * `base64_bytes` - A base64-encoded string representing the document's binary data.
///                    Must be valid base64 encoding using the standard alphabet.
///
/// * `filename` - Optional filename including extension (e.g., "document.pdf").
///                The extension is used as a hint for file type detection if provided.
///                If `None`, the file type will be detected from the content alone.
///
/// * `options_json` - Optional JSON string containing estimation options.
///                    Should deserialize to an `EstimateOptions` struct.
///                    If `None` or invalid JSON, default options will be used.
///
/// # Returns
///
/// A `JsValue` containing a JSON string with the estimation result or error.
///
/// On success, returns a JSON object with fields:
/// - `pages`: Estimated page count
/// - `format`: Detected document format
/// - Additional format-specific metadata
///
/// On failure, returns a JSON object with an `error` field describing the issue.
///
/// # Example
///
/// ```javascript
/// // From JavaScript
/// const base64Data = btoa(fileData);
/// const result = estimate_document_base64(base64Data, "report.pdf", null);
/// const estimate = JSON.parse(result);
/// console.log(`Estimated pages: ${estimate.pages}`);
/// ```
///
/// # Errors
///
/// Returns an error object if:
/// - The base64 string cannot be decoded
/// - The document format is unsupported
/// - The document is corrupted or invalid
#[wasm_bindgen]
pub fn estimate_document_base64(
    base64_bytes: &str,
    filename: Option<String>,
    options_json: Option<String>,
) -> JsValue {
    // convenience wrapper to allow passing base64 bytes from JS (where typed arrays may not be handy)
    match base64::engine::general_purpose::STANDARD.decode(base64_bytes) {
        Ok(bytes) => estimate_document(&bytes, filename, options_json),
        Err(e) => JsValue::from_str(
            &json!({"error": format!("base64 decode failed: {:?}", e)}).to_string(),
        ),
    }
}

/// Estimates the number of pages in a document from raw byte data.
///
/// This is the core estimation function that processes document bytes directly.
/// It automatically detects the document type, applies the appropriate estimation
/// algorithm, and returns a structured result.
///
/// # Parameters
///
/// * `bytes` - A byte slice containing the raw document data. This should be the
///             complete binary content of the document file.
///
/// * `filename` - Optional filename including extension (e.g., "spreadsheet.xlsx").
///                The extension helps with file type detection but is not required.
///                If `None`, type detection relies solely on content analysis
///                (magic bytes, file signatures, etc.).
///
/// * `options_json` - Optional JSON string containing estimation configuration.
///                    Should match the `EstimateOptions` struct schema.
///                    If `None` or parsing fails, default options are applied.
///                    Default options typically use standard page dimensions and
///                    conservative estimation heuristics.
///
/// # Returns
///
/// A `JsValue` containing a JSON string with the estimation result or error details.
///
/// ## Success Response
///
/// Returns a JSON object containing:
/// - `pages` (number): The estimated page count
/// - `format` (string): Detected document format ("pdf", "xlsx", "txt", "markdown")
/// - `confidence` (optional number): Estimation confidence score
/// - Additional format-specific fields (e.g., sheet count for XLSX)
///
/// ## Error Response
///
/// Returns a JSON object containing:
/// - `error` (string): Human-readable error message
/// - `detected` (string): The detected format (for debugging)
///
/// # Type Detection
///
/// The function uses a multi-stage detection process:
/// 1. File extension (if filename provided)
/// 2. Magic bytes / file signatures
/// 3. Content structure analysis
///
/// # Supported Formats
///
/// - **PDF**: Counts pages using PDF structure markers (`/Type /Page`)
/// - **XLSX**: Counts worksheets in the Excel workbook
/// - **TXT**: Estimates based on character count, line breaks, and page size settings
/// - **Markdown**: Estimates considering markdown syntax and rendered output
///
/// # Example
///
/// ```javascript
/// // From JavaScript with Uint8Array
/// const fileBytes = new Uint8Array(fileData);
/// const optionsJson = JSON.stringify({ pageWidth: 8.5, pageHeight: 11 });
/// const result = estimate_document(fileBytes, "document.pdf", optionsJson);
/// const data = JSON.parse(result);
///
/// if (data.error) {
///     console.error(`Error: ${data.error}`);
/// } else {
///     console.log(`Document has approximately ${data.pages} pages`);
/// }
/// ```
///
/// # Errors
///
/// Returns an error object if:
/// - The document format is not supported
/// - The document structure is invalid or corrupted
/// - Required content markers are missing or malformed
/// - The document is empty or truncated
///
/// # Performance
///
/// For large documents, the estimation is optimized to avoid full parsing when possible.
/// PDF page counting uses regex pattern matching rather than full PDF parsing.
#[wasm_bindgen]
pub fn estimate_document(
    bytes: &[u8],
    filename: Option<String>,
    options_json: Option<String>,
) -> JsValue {
    // parse options
    let options: EstimateOptions = match options_json {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => EstimateOptions::default(),
    };

    let detected = detect_type(filename.as_deref(), bytes);

    let result = match detected.as_str() {
        "pdf" => match estimate_pdf_pages(bytes, &options) {
            Ok(r) => Ok(r),
            Err(err) => Err(err.to_string()),
        },
        "xlsx" => match estimate_xlsx_pages(bytes, &options) {
            Ok(r) => Ok(r),
            Err(err) => Err(err.to_string()),
        },
        "txt" => Ok(estimate_text_pages(bytes, &options)),
        "markdown" => Ok(estimate_markdown_pages(bytes, &options)),
        other => Err(format!("Unsupported or unrecognized format: {}", other)),
    };

    match result {
        Ok(est) => match serde_json::to_string(&est) {
            Ok(s) => JsValue::from_str(&s),
            Err(_) => JsValue::from_str(&json!({"error":"serialization failed"}).to_string()),
        },
        Err(err_msg) => {
            JsValue::from_str(&json!({"error": err_msg, "detected": detected}).to_string())
        }
    }
}
