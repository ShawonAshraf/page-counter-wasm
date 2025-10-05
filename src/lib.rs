mod estimators;
mod file_utils;
mod schema;

use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

// XLSX parsing
use base64::Engine;
use calamine::Reader;
use estimators::{
    estimate_markdown_pages, estimate_pdf_pages, estimate_text_pages, estimate_xlsx_pages,
};
use file_utils::detect_type;
use schema::{EstimateOptions, PageSizeMm};

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
