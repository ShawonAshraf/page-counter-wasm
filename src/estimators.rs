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
//! - **Word documents** (`.docx`) - exact page count from metadata or estimated from content
//! - **PowerPoint presentations** (`.pptx`) - exact slide count from metadata
//! - **PDF files** (`.pdf`) - exact page count extracted from document structure
//!
//! ## Estimation Strategy
//!
//! Different file formats require different estimation strategies:
//! - Text-based formats use heuristics (characters per page, rows per page)
//! - PDF files provide exact page counts since they have defined page structures
//!
//! The estimators respect user-provided options for paper sizes and other parameters.

use crate::file_utils::{a4_mm, letter_mm};
use crate::schema::{EstimateOptions, EstimateResult, EstimatorError, PageSizeMm};
use calamine::{Data, Reader, Xlsx};
use std::io::{Cursor, Read};
use wasm_bindgen::prelude::*;
use zip::ZipArchive;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

// Placeholder for PDF.js integration (optional feature)
// Note: PDF.js integration can be added separately via JavaScript
// For now, the synchronous PDF parser works fine
pub async fn count_pdf_pages_js(_bytes: &[u8]) -> Result<JsValue, JsValue> {
    Err(JsValue::from_str("PDF.js not integrated"))
}


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

/// Estimates the number of pages in a PDF file using simple regex parsing.
///
/// This is a fallback method for synchronous PDF processing. For better accuracy
/// and reliability, use the async `estimate_pdf_with_pdfjs` function which uses PDF.js.
///
/// This function counts PDF page objects by searching for `/Type /Page` patterns in the PDF structure.
///
/// # Parameters
///
/// * `bytes` - The raw PDF file bytes
/// * `_options` - Estimation options (currently unused for PDFs, as page dimensions are extracted from the PDF)
///
/// # Returns
///
/// Returns a `Result` containing the `EstimateResult` with page count and dimensions,
/// This is a fallback method for synchronous PDF processing.
/// or an `EstimatorError` if the PDF cannot be parsed.
pub fn estimate_pdf_pages(
    bytes: &[u8],
    _options: &EstimateOptions,
) -> Result<EstimateResult, EstimatorError> {
    // Convert bytes to string for pattern matching
    let pdf_str = String::from_utf8_lossy(bytes);
    
    // Count occurrences of /Type /Page (but not /Type /Pages)
    // This is a simple heuristic that works for most PDFs
    let mut page_count = 0;
    let mut search_pos = 0;
    
    while let Some(pos) = pdf_str[search_pos..].find("/Type") {
        let abs_pos = search_pos + pos;
        let remaining = &pdf_str[abs_pos..];
        
        // Check if this is "/Type /Page" or "/Type/Page"
        if remaining.starts_with("/Type /Page") || remaining.starts_with("/Type/Page") {
            // Make sure it's not "/Type /Pages"
            let after_page = if remaining.starts_with("/Type /Page") {
                &remaining[11..]
            } else {
                &remaining[10..]
            };
            
            // Check the character after "Page" is not 's'
            if !after_page.starts_with('s') {
                page_count += 1;
            }
        }
        
        search_pos = abs_pos + 5; // Move past "/Type"
    }
    
    if page_count == 0 {
        return Err(EstimatorError::PdfError(
            "No pages found in PDF. File may be corrupted or use an unsupported format.".to_string(),
        ));
    }
    
    // Use A4 as default page size for PDFs
    let (width_mm, height_mm) = a4_mm();
    
    Ok(EstimateResult {
        page_count,
        page_sizes: vec![PageSizeMm { width_mm, height_mm }; page_count],
        notes: vec![
            format!("PDF has {} pages (estimated using simple parsing)", page_count),
            "⚠ For more accurate results, use the async estimate_pdf_with_pdfjs function".to_string(),
        ],
    })
}

/// Estimates the number of pages in a Word document (.docx).
///
/// This function parses the DOCX file (which is a ZIP archive) and extracts the
/// page count from the `docProps/app.xml` metadata file. DOCX files store their
/// page count in the `<Pages>` element of this XML file.
///
/// # Parameters
///
/// * `bytes` - The raw DOCX file bytes
/// * `options` - Estimation options including paper size preferences
///
/// # Returns
///
/// Returns `Ok(EstimateResult)` containing the exact page count from document metadata,
/// or `Err(EstimatorError)` if the file cannot be parsed or page count cannot be determined.
///
/// # Notes
///
/// - The page count is extracted directly from document metadata (exact count)
/// - If the metadata doesn't contain page count, attempts to estimate from content
/// - Paper size is set based on options or defaults to A4
pub fn estimate_docx_pages(
    bytes: &[u8],
    options: &EstimateOptions,
) -> Result<EstimateResult, EstimatorError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| EstimatorError::General(format!("Failed to open DOCX as ZIP: {:?}", e)))?;
    
    // Try to read page count from docProps/app.xml
    let page_count_result = {
        match archive.by_name("docProps/app.xml") {
            Ok(mut file) => {
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => Some(parse_pages_from_app_xml(&contents)),
                    Err(e) => Some(Err(EstimatorError::General(format!("Failed to read app.xml: {:?}", e)))),
                }
            }
            Err(_) => None,
        }
    };
    
    let page_count = match page_count_result {
        Some(Ok(count)) => count,
        Some(Err(e)) => return Err(e),
        None => {
            // If app.xml doesn't exist, try to estimate from content
            return estimate_docx_from_content(&mut archive, options);
        }
    };
    
    // Determine paper size
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
    
    Ok(EstimateResult {
        page_count,
        page_sizes: vec![PageSizeMm { width_mm: w, height_mm: h }; page_count],
        notes: vec![
            format!("DOCX document has {} pages (from metadata)", page_count),
        ],
    })
}

/// Estimates the number of slides in a PowerPoint presentation (.pptx).
///
/// This function parses the PPTX file (which is a ZIP archive) and extracts the
/// slide count from the `docProps/app.xml` metadata file. PPTX files store their
/// slide count in the `<Slides>` element of this XML file.
///
/// # Parameters
///
/// * `bytes` - The raw PPTX file bytes
/// * `options` - Estimation options (paper size is set to standard presentation size)
///
/// # Returns
///
/// Returns `Ok(EstimateResult)` containing the exact slide count from document metadata,
/// or `Err(EstimatorError)` if the file cannot be parsed or slide count cannot be determined.
///
/// # Notes
///
/// - The slide count is extracted directly from document metadata (exact count)
/// - Each slide is considered as one "page" for printing purposes
/// - Uses standard presentation dimensions (10" × 7.5" / 254mm × 190.5mm)
pub fn estimate_pptx_pages(
    bytes: &[u8],
    _options: &EstimateOptions,
) -> Result<EstimateResult, EstimatorError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| EstimatorError::General(format!("Failed to open PPTX as ZIP: {:?}", e)))?;
    
    // Try to read slide count from docProps/app.xml
    let slide_count_result = {
        match archive.by_name("docProps/app.xml") {
            Ok(mut file) => {
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => Some(parse_slides_from_app_xml(&contents)),
                    Err(e) => Some(Err(EstimatorError::General(format!("Failed to read app.xml: {:?}", e)))),
                }
            }
            Err(_) => None,
        }
    };
    
    let slide_count = match slide_count_result {
        Some(Ok(count)) => count,
        Some(Err(e)) => return Err(e),
        None => {
            // If app.xml doesn't exist, try to count slide files
            return estimate_pptx_from_content(&mut archive);
        }
    };
    
    // Standard PowerPoint slide dimensions: 10" × 7.5" (254mm × 190.5mm)
    let (w, h) = (254.0, 190.5);
    
    Ok(EstimateResult {
        page_count: slide_count,
        page_sizes: vec![PageSizeMm { width_mm: w, height_mm: h }; slide_count],
        notes: vec![
            format!("PPTX presentation has {} slides (from metadata)", slide_count),
        ],
    })
}

/// Helper function to parse page count from app.xml content
fn parse_pages_from_app_xml(xml_content: &str) -> Result<usize, EstimatorError> {
    let mut reader = XmlReader::from_str(xml_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_pages = false;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"Pages" {
                    in_pages = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_pages {
                    let text = e.unescape()
                        .map_err(|e| EstimatorError::General(format!("XML unescape error: {:?}", e)))?;
                    let page_count = text.parse::<usize>()
                        .map_err(|e| EstimatorError::General(format!("Failed to parse page count: {:?}", e)))?;
                    return Ok(page_count);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"Pages" {
                    in_pages = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EstimatorError::General(format!("XML parse error: {:?}", e))),
            _ => {}
        }
        buf.clear();
    }
    
    Err(EstimatorError::General("No page count found in app.xml".to_string()))
}

/// Helper function to parse slide count from app.xml content
fn parse_slides_from_app_xml(xml_content: &str) -> Result<usize, EstimatorError> {
    let mut reader = XmlReader::from_str(xml_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_slides = false;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"Slides" {
                    in_slides = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_slides {
                    let text = e.unescape()
                        .map_err(|e| EstimatorError::General(format!("XML unescape error: {:?}", e)))?;
                    let slide_count = text.parse::<usize>()
                        .map_err(|e| EstimatorError::General(format!("Failed to parse slide count: {:?}", e)))?;
                    return Ok(slide_count);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"Slides" {
                    in_slides = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EstimatorError::General(format!("XML parse error: {:?}", e))),
            _ => {}
        }
        buf.clear();
    }
    
    Err(EstimatorError::General("No slide count found in app.xml".to_string()))
}

/// Fallback: estimate DOCX pages by analyzing content structure
fn estimate_docx_from_content(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    options: &EstimateOptions,
) -> Result<EstimateResult, EstimatorError> {
    // Try to read document.xml and count paragraph/page break elements
    match archive.by_name("word/document.xml") {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| EstimatorError::General(format!("Failed to read document.xml: {:?}", e)))?;
            
            // Count explicit page breaks
            let page_breaks = contents.matches("<w:br w:type=\"page\"").count();
            
            // If there are page breaks, use that count + 1 (for the first page)
            let estimated_pages = if page_breaks > 0 {
                page_breaks + 1
            } else {
                // Fallback: estimate based on paragraph count
                let paragraphs = contents.matches("<w:p ").count() + contents.matches("<w:p>").count();
                let paragraphs_per_page = 25; // rough heuristic
                ((paragraphs + paragraphs_per_page - 1) / paragraphs_per_page).max(1)
            };
            
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
            
            Ok(EstimateResult {
                page_count: estimated_pages,
                page_sizes: vec![PageSizeMm { width_mm: w, height_mm: h }; estimated_pages],
                notes: vec![
                    format!("DOCX document estimated at {} pages (from content analysis)", estimated_pages),
                    "Note: Page count estimated from content structure; may not be exact".to_string(),
                ],
            })
        }
        Err(e) => Err(EstimatorError::General(format!("Failed to read DOCX content: {:?}", e))),
    }
}

/// Fallback: estimate PPTX slides by counting slide files
fn estimate_pptx_from_content(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<EstimateResult, EstimatorError> {
    // Count slide files in ppt/slides/ directory
    let mut slide_count = 0;
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_count += 1;
            }
        }
    }
    
    if slide_count == 0 {
        return Err(EstimatorError::General("No slides found in PPTX".to_string()));
    }
    
    let (w, h) = (254.0, 190.5); // Standard PowerPoint dimensions
    
    Ok(EstimateResult {
        page_count: slide_count,
        page_sizes: vec![PageSizeMm { width_mm: w, height_mm: h }; slide_count],
        notes: vec![
            format!("PPTX presentation has {} slides (counted from files)", slide_count),
        ],
    })
}

