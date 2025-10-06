//! JavaScript bindings for PDF.js
//!
//! This module provides Rust bindings to call PDF.js functions from WASM.
//! PDF.js is much faster and more reliable than parsing PDFs in pure Rust/WASM.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/pdfjs_bridge.js")]
extern "C" {
    /// Call PDF.js to count pages in a PDF
    /// Returns a JSON string with: {page_count: number, width_pt: number, height_pt: number}
    /// Returns null if PDF.js fails or is not available
    #[wasm_bindgen(catch)]
    pub async fn count_pdf_pages_js(bytes: &[u8]) -> Result<JsValue, JsValue>;
}

