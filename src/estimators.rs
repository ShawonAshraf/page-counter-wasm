use crate::file_utils::{a4_mm, letter_mm, mm_from_pt, obj_to_f64};
use crate::schema::{EstimateOptions, EstimateResult, EstimatorError, PageSizeMm};
use calamine::Xlsx;
use std::io::Cursor;

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

pub fn estimate_markdown_pages(bytes: &[u8], options: &EstimateOptions) -> EstimateResult {
    // for now treat markdown similar to text (could parse headings and images later)
    let mut res = estimate_text_pages(bytes, options);
    res.notes
        .push("Markdown parsed as text; images/embedded content not considered.".into());
    res
}

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

pub fn estimate_pdf_pages(
    bytes: &[u8],
    _options: &EstimateOptions,
) -> Result<EstimateResult, EstimatorError> {
    // parse with lopdf
    let doc =
        LopdfDocument::load_mem(bytes).map_err(|e| EstimatorError::PdfError(format!("{:?}", e)))?;

    let pages_tree = doc.get_pages(); // BTreeMap of page_num -> object id
    let mut page_sizes = Vec::new();
    let mut notes = Vec::new();

    for (pnum, page_id) in pages_tree.iter() {
        // We'll try to read MediaBox or CropBox if possible via the page object
        // We'll attempt retrieving the object and extracting MediaBox or use default 595x842 points (A4 ~ 595x842 points)
        // Safe fallback values:
        let default_pts = (595.0_f64, 842.0_f64); // ~A4 in points (close)
        let mut width_pts = default_pts.0;
        let mut height_pts = default_pts.1;

        // try to get MediaBox from the page dictionary
        if let Ok(obj) = doc.get_object(*page_id) {
            if let lopdf::Object::Dictionary(page_dict) = obj {
                if let Ok(mediabox_obj) = page_dict.get(b"MediaBox") {
                    if let lopdf::Object::Array(vals) = mediabox_obj {
                        // array of four numbers: [llx, lly, urx, ury]
                        if vals.len() == 4 {
                            // attempt converting to f64
                            let x0 = obj_to_f64(&vals[0]);
                            let y0 = obj_to_f64(&vals[1]);
                            let x1 = obj_to_f64(&vals[2]);
                            let y1 = obj_to_f64(&vals[3]);
                            width_pts = (x1 - x0).abs();
                            height_pts = (y1 - y0).abs();
                        }
                    }
                } else if let Ok(cropbox_obj) = page_dict.get(b"CropBox") {
                    if let lopdf::Object::Array(vals) = cropbox_obj {
                        if vals.len() == 4 {
                            let x0 = obj_to_f64(&vals[0]);
                            let y0 = obj_to_f64(&vals[1]);
                            let x1 = obj_to_f64(&vals[2]);
                            let y1 = obj_to_f64(&vals[3]);
                            width_pts = (x1 - x0).abs();
                            height_pts = (y1 - y0).abs();
                        }
                    }
                }
            }
        }

        let width_mm = mm_from_pt(width_pts);
        let height_mm = mm_from_pt(height_pts);
        page_sizes.push(PageSizeMm {
            width_mm,
            height_mm,
        });
        notes.push(format!(
            "Page {}: {:.2} x {:.2} mm",
            *pnum as usize, width_mm, height_mm
        ));
    }

    let page_count = page_sizes.len();

    Ok(EstimateResult {
        page_count,
        page_sizes,
        notes,
    })
}
