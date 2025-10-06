//! # Page Count Estimators
//!
//! This module provides estimation functions for various document formats.
//! Each estimator analyzes the input bytes and returns an estimated page count
//! along with page dimensions and processing notes.
//!
//! ## Supported Formats
//!
//! - **Text files** (`.txt`) - estimated based on character count
//! - **Markdown files** (`.md`) - treated similarly to text files
//! - **Excel files** (`.xlsx`) - estimated based on row count per sheet
//! - **PDF files** (`.pdf`) - exact page count extracted from document structure
//!
//! ## Estimation Strategy
//!
//! Different file formats require different estimation strategies:
//! - Text-based formats use heuristics (characters per page, rows per page)
//! - PDF files provide exact page counts since they have defined page structures
//!
//! The estimators respect user-provided options for paper sizes and other parameters.

use crate::file_utils::{a4_mm, letter_mm, mm_from_pt, obj_to_f64};
use crate::schema::{EstimateOptions, EstimateResult, EstimatorError, PageSizeMm};
use calamine::{Data, Reader, Xlsx};
use lopdf::Document as LopdfDocument;
use std::io::Cursor;

/// Estimates the number of pages for a plain text file.
///
/// This function uses a character-based heuristic to estimate how many pages
/// the text would occupy when printed. The estimation assumes a fixed number
/// of characters per page (default: 1800 characters).
///
/// # Arguments
///
/// * `bytes` - The raw bytes of the text file
/// * `options` - Estimation options including:
///   - `chars_per_page`: Number of characters per page (default: 1800)
///   - `default_paper`: Paper size ("Letter" or "A4")
///   - `custom_paper_mm`: Custom paper dimensions in millimeters
///
/// # Returns
///
/// Returns an `EstimateResult` containing:
/// - `page_count`: Estimated number of pages
/// - `page_sizes`: Vector of page dimensions (all pages have the same size)
/// - `notes`: Processing information including character count and chars per page
///
/// # Notes
///
/// - If the input is not valid UTF-8, returns 0 pages with an error note
/// - The character count is based on Unicode characters, not bytes
/// - Pages are rounded up (e.g., 1801 characters = 2 pages with default settings)
///
/// # Example
///
/// ```ignore
/// let options = EstimateOptions {
///     chars_per_page: Some(2000),
///     default_paper: Some("A4".to_string()),
///     ..Default::default()
/// };
/// let result = estimate_text_pages(file_bytes, &options);
/// println!("Estimated {} pages", result.page_count);
/// ```
pub fn estimate_text_pages(bytes: &[u8], options: &EstimateOptions) -> EstimateResult {
    let s = match std::str::from_utf8(bytes) {
        Ok(v) => v,
        Err(_) => {
            return EstimateResult {
                page_count: 0,
                page_sizes: vec![],
                notes: vec!["Text not valid UTF-8".into()],
            };
        }
    };

    let chars = s.chars().count();
    let chars_per_page = options.chars_per_page.unwrap_or(1800); // heuristic ~ 1800 chars/page
    let pages = (chars + chars_per_page - 1) / chars_per_page;

    // decide paper size
    let (w, h) = if let Some(custom) = options.custom_paper_mm {
        custom
    } else if let Some(ref def) = options.default_paper {
        match def.as_str() {
            "Letter" | "letter" => letter_mm(),
            _ => a4_mm(),
        }
    } else {
        a4_mm()
    };

    let mut notes = Vec::new();
    notes.push(format!(
        "chars: {}, chars_per_page: {}",
        chars, chars_per_page
    ));

    EstimateResult {
        page_count: pages,
        page_sizes: vec![
            PageSizeMm {
                width_mm: w,
                height_mm: h
            };
            pages
        ],
        notes,
    }
}

/// Estimates the number of pages for a Markdown file.
///
/// Currently, this function treats Markdown files similarly to plain text files,
/// using the same character-based estimation. Future versions may parse Markdown
/// structure (headings, code blocks, images) for more accurate estimates.
///
/// # Arguments
///
/// * `bytes` - The raw bytes of the Markdown file
/// * `options` - Estimation options (same as `estimate_text_pages`)
///
/// # Returns
///
/// Returns an `EstimateResult` similar to text estimation, with an additional
/// note indicating that the file was parsed as plain text.
///
/// # Limitations
///
/// - Images and embedded content are not considered in the estimation
/// - Markdown formatting (headings, lists, code blocks) is not accounted for
/// - The estimation is purely based on character count
///
/// # Example
///
/// ```ignore
/// let result = estimate_markdown_pages(markdown_bytes, &options);
/// // Returns same estimation as plain text with additional note
/// ```
pub fn estimate_markdown_pages(bytes: &[u8], options: &EstimateOptions) -> EstimateResult {
    // for now treat markdown text similar to text (could parse headings and images later)
    let mut res = estimate_text_pages(bytes, options);
    res.notes
        .push("Markdown parsed as text; images/embedded content not considered.".into());
    res
}

/// Estimates the number of pages for an Excel (.xlsx) file.
///
/// This function parses the Excel workbook and estimates pages based on the number
/// of non-empty rows in each worksheet. Each sheet is processed independently, and
/// the total page count is the sum of pages across all sheets.
///
/// # Arguments
///
/// * `bytes` - The raw bytes of the Excel file
/// * `options` - Estimation options including:
///   - `rows_per_page`: Number of rows per printed page (default: 40)
///   - `default_paper`: Paper size ("Letter" or "A4")
///   - `custom_paper_mm`: Custom paper dimensions in millimeters
///
/// # Returns
///
/// Returns `Ok(EstimateResult)` on success, containing:
/// - `page_count`: Total estimated pages across all sheets
/// - `page_sizes`: Vector of page dimensions for each page
/// - `notes`: Detailed information about each sheet (row count, page count)
///
/// Returns `Err(EstimatorError::XlsxError)` if the file cannot be parsed.
///
/// # Sheet Processing
///
/// - Only non-empty rows are counted (rows with at least one non-empty cell)
/// - Empty sheets contribute 0 pages to the total
/// - Unreadable sheets are noted but don't cause the estimation to fail
///
/// # Example
///
/// ```ignore
/// let options = EstimateOptions {
///     rows_per_page: Some(50),
///     default_paper: Some("Letter".to_string()),
///     ..Default::default()
/// };
/// match estimate_xlsx_pages(excel_bytes, &options) {
///     Ok(result) => println!("Estimated {} pages", result.page_count),
///     Err(e) => eprintln!("Failed to parse Excel file: {:?}", e),
/// }
/// ```
pub fn estimate_xlsx_pages(
    bytes: &[u8],
    options: &EstimateOptions,
) -> Result<EstimateResult, EstimatorError> {
    let cursor = Cursor::new(bytes);
    let mut xlsx = Xlsx::new(cursor).map_err(|e| EstimatorError::XlsxError(format!("{:?}", e)))?;
    let rows_per_page = options.rows_per_page.unwrap_or(40); // heuristic
    let (w, h) = if let Some(custom) = options.custom_paper_mm {
        custom
    } else if let Some(ref def) = options.default_paper {
        match def.as_str() {
            "Letter" | "letter" => letter_mm(),
            _ => a4_mm(),
        }
    } else {
        a4_mm()
    };

    let mut total_pages = 0usize;
    let mut notes = Vec::new();
    let mut per_page_sizes = Vec::new();

    for sheet_name in xlsx.sheet_names().to_owned() {
        match xlsx.worksheet_range(&sheet_name) {
            Ok(range) => {
                // count non-empty rows
                let mut last_row_index = 0usize;
                for (ridx, row) in range.rows().enumerate() {
                    // treat row as non-empty if any cell non-empty
                    if row.iter().any(|c| !matches!(c, Data::Empty)) {
                        last_row_index = ridx + 1;
                    }
                }
                let pages_for_sheet = (last_row_index + rows_per_page - 1) / rows_per_page;
                if pages_for_sheet > 0 {
                    total_pages += pages_for_sheet;
                    per_page_sizes.extend(
                        std::iter::repeat(PageSizeMm {
                            width_mm: w,
                            height_mm: h,
                        })
                        .take(pages_for_sheet),
                    );
                    notes.push(format!(
                        "Sheet '{}' rows: {}, pages: {}",
                        sheet_name, last_row_index, pages_for_sheet
                    ));
                } else {
                    notes.push(format!("Sheet '{}' empty; 0 pages", sheet_name));
                }
            }
            Err(_) => {
                notes.push(format!("Could not read sheet '{}'", sheet_name));
            }
        }
    }

    if total_pages == 0 {
        // maybe workbook is empty
        notes.push("Workbook appears empty or unreadable; returning 0 pages.".into());
    }

    Ok(EstimateResult {
        page_count: total_pages,
        page_sizes: per_page_sizes,
        notes,
    })
}

/// Extracts the exact page count and page dimensions from a PDF file.
///
/// This optimized implementation quickly counts pages and extracts dimensions efficiently.
/// For large PDFs with uniform page sizes, it samples the first page's dimensions
/// rather than parsing every page individually, resulting in significantly better performance.
///
/// # Arguments
///
/// * `bytes` - The raw bytes of the PDF file
/// * `_options` - Estimation options (currently unused; PDF dimensions are extracted from the file)
///
/// # Returns
///
/// Returns `Ok(EstimateResult)` on success, containing:
/// - `page_count`: Exact number of pages in the PDF
/// - `page_sizes`: Vector of page dimensions (in millimeters)
/// - `notes`: Information about page dimensions and optimization applied
///
/// Returns `Err(EstimatorError::PdfError)` if the PDF cannot be parsed.
///
/// # Performance Optimization
///
/// This function uses a fast-path optimization:
/// 1. Quickly extracts the page count from the page tree
/// 2. Extracts dimensions from only the first page (and optionally a sample of later pages)
/// 3. Assumes uniform page sizes (typical for most PDFs)
/// 4. Falls back to per-page extraction only if mixed sizes are detected
///
/// This approach is **much faster** for large PDFs compared to extracting dimensions
/// from every page individually.
///
/// # Page Dimensions
///
/// The function attempts to extract page dimensions in the following order:
/// 1. **MediaBox**: The primary bounding box for the page
/// 2. **CropBox**: The visible area of the page (used if MediaBox is not available)
/// 3. **Default**: Falls back to A4 size (595×842 points ≈ 210×297 mm) if neither is found
///
/// Dimensions are converted from PDF points (1/72 inch) to millimeters.
///
/// # Example
///
/// ```ignore
/// match estimate_pdf_pages(pdf_bytes, &options) {
///     Ok(result) => {
///         println!("PDF has {} pages", result.page_count);
///         println!("First page: {:.1} × {:.1} mm", 
///                  result.page_sizes[0].width_mm, result.page_sizes[0].height_mm);
///     },
///     Err(e) => eprintln!("Failed to parse PDF: {:?}", e),
/// }
/// ```
pub fn estimate_pdf_pages(
    bytes: &[u8],
    _options: &EstimateOptions,
) -> Result<EstimateResult, EstimatorError> {
    // parse with lopdf
    let doc =
        LopdfDocument::load_mem(bytes).map_err(|e| EstimatorError::PdfError(format!("{:?}", e)))?;

    let pages_tree = doc.get_pages(); // BTreeMap of page_num -> object id
    let page_count = pages_tree.len();
    
    if page_count == 0 {
        return Ok(EstimateResult {
            page_count: 0,
            page_sizes: vec![],
            notes: vec!["PDF has no pages".to_string()],
        });
    }

    let mut notes = Vec::new();
    
    // OPTIMIZATION: For large PDFs, sample dimensions from first page only
    // Most PDFs have uniform page sizes, so this is much faster than iterating all pages
    let sample_size = if page_count > 100 { 1 } else { page_count.min(5) };
    
    // Extract dimensions from first page (and optionally a few more)
    let sampled_dims: Vec<(f64, f64)> = pages_tree
        .iter()
        .take(sample_size)
        .map(|(_, page_id)| extract_page_dimensions(&doc, *page_id))
        .collect();
    
    // Check if all sampled pages have the same dimensions (within tolerance)
    let first_dim = sampled_dims[0];
    let uniform = sampled_dims.iter().all(|(w, h)| {
        (w - first_dim.0).abs() < 0.1 && (h - first_dim.1).abs() < 0.1
    });
    
    let page_sizes = if uniform {
        // Fast path: All pages use the same dimensions
        let width_mm = mm_from_pt(first_dim.0);
        let height_mm = mm_from_pt(first_dim.1);
        
        notes.push(format!(
            "PDF has {} pages (uniform size: {:.1} × {:.1} mm)",
            page_count, width_mm, height_mm
        ));
        
        if page_count > 100 {
            notes.push("Performance optimization: Sampled first page only (large PDF)".to_string());
        }
        
        vec![PageSizeMm { width_mm, height_mm }; page_count]
    } else {
        // Fallback: Mixed page sizes detected, extract all dimensions
        notes.push(format!(
            "PDF has {} pages with mixed sizes",
            page_count
        ));
        
        pages_tree
            .iter()
            .map(|(pnum, page_id)| {
                let (width_pts, height_pts) = extract_page_dimensions(&doc, *page_id);
                let width_mm = mm_from_pt(width_pts);
                let height_mm = mm_from_pt(height_pts);
                
                if page_count <= 20 {
                    notes.push(format!(
                        "Page {}: {:.1} × {:.1} mm",
                        pnum, width_mm, height_mm
                    ));
                }
                
                PageSizeMm { width_mm, height_mm }
            })
            .collect()
    };

    Ok(EstimateResult {
        page_count,
        page_sizes,
        notes,
    })
}

/// Helper function to extract page dimensions (width, height) in points.
///
/// Attempts to read MediaBox or CropBox from the page dictionary.
/// Falls back to A4 size (595×842 points) if dimensions cannot be extracted.
fn extract_page_dimensions(doc: &LopdfDocument, page_id: (u32, u16)) -> (f64, f64) {
    let default_pts = (595.0_f64, 842.0_f64); // A4 in points
    let mut width_pts = default_pts.0;
    let mut height_pts = default_pts.1;

    if let Ok(obj) = doc.get_object(page_id) {
        if let lopdf::Object::Dictionary(page_dict) = obj {
            // Try MediaBox first
            if let Ok(mediabox_obj) = page_dict.get(b"MediaBox") {
                if let Some((w, h)) = parse_box_dimensions(mediabox_obj) {
                    width_pts = w;
                    height_pts = h;
                }
            } else if let Ok(cropbox_obj) = page_dict.get(b"CropBox") {
                // Fallback to CropBox
                if let Some((w, h)) = parse_box_dimensions(cropbox_obj) {
                    width_pts = w;
                    height_pts = h;
                }
            }
        }
    }

    (width_pts, height_pts)
}

/// Helper function to parse a PDF box (MediaBox or CropBox) into (width, height).
fn parse_box_dimensions(box_obj: &lopdf::Object) -> Option<(f64, f64)> {
    if let lopdf::Object::Array(vals) = box_obj {
        if vals.len() == 4 {
            let x0 = obj_to_f64(&vals[0]);
            let y0 = obj_to_f64(&vals[1]);
            let x1 = obj_to_f64(&vals[2]);
            let y1 = obj_to_f64(&vals[3]);
            let width = (x1 - x0).abs();
            let height = (y1 - y0).abs();
            return Some((width, height));
        }
    }
    None
}
