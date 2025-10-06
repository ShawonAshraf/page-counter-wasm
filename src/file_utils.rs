/// Converts points to millimeters.
///
/// # Arguments
///
/// * `pt` - The value in points to convert
///
/// # Returns
///
/// The equivalent value in millimeters. Conversion is based on:
/// - 1 point = 1/72 inch
/// - 1 inch = 25.4 mm
///
/// # Example
///
/// ```
/// let mm = mm_from_pt(72.0); // 72 points = 1 inch = 25.4 mm
/// ```
pub fn mm_from_pt(pt: f64) -> f64 {
    // 1 point = 1/72 inch; 1 inch = 25.4 mm
    pt / 72.0 * 25.4
}

/// Returns the standard A4 paper dimensions in millimeters.
///
/// # Returns
///
/// A tuple `(width, height)` representing the A4 dimensions:
/// - Width: 210.0 mm
/// - Height: 297.0 mm
pub fn a4_mm() -> (f64, f64) {
    (210.0, 297.0)
}

/// Returns the standard Letter paper dimensions in millimeters.
///
/// # Returns
///
/// A tuple `(width, height)` representing the Letter dimensions:
/// - Width: 215.9 mm (8.5 inches)
/// - Height: 279.4 mm (11 inches)
pub fn letter_mm() -> (f64, f64) {
    (215.9, 279.4)
}

/// Detects the file type from filename extension or magic bytes.
///
/// This function attempts to identify the file type by first checking the filename
/// extension (if provided), and then falling back to examining the file's magic bytes.
///
/// # Arguments
///
/// * `filename` - Optional filename to check for extension-based detection
/// * `bytes` - The file contents as a byte slice for magic byte detection
///
/// # Returns
///
/// A string representing the detected file type:
/// - `"pdf"` - PDF documents (detected by .pdf extension or %PDF magic bytes)
/// - `"xlsx"` - Excel spreadsheets (detected by .xlsx/.xlsm extension or PK zip signature)
/// - `"markdown"` - Markdown files (detected by .md/.markdown extension)
/// - `"txt"` - Plain text files (detected by .txt extension or printable ASCII content)
/// - `"unknown"` - Unable to determine file type
///
/// # Detection Strategy
///
/// 1. Filename extension check (case-insensitive)
/// 2. Magic bytes check for PDF (%PDF header)
/// 3. Magic bytes check for ZIP-based formats (PK signature, assumed to be xlsx)
/// 4. Text detection based on printable ASCII characters (tabs, newlines, spaces, and chars 32-127)
pub fn detect_type(filename: Option<&str>, bytes: &[u8]) -> String {
    if let Some(name) = filename {
        let lower = name.to_lowercase();
        if lower.ends_with(".pdf") {
            return "pdf".into();
        }
        if lower.ends_with(".xlsx") || lower.ends_with(".xlsm") {
            return "xlsx".into();
        }
        if lower.ends_with(".md") || lower.ends_with(".markdown") {
            return "markdown".into();
        }
        if lower.ends_with(".txt") {
            return "txt".into();
        }
    }
    // fallback: magic
    if bytes.len() >= 4 && &bytes[0..4] == b"%PDF" {
        return "pdf".into();
    }
    // xlsx is a zip with PK
    if bytes.len() >= 4 && &bytes[0..2] == b"PK" {
        // assume xlsx (could be other zip types)
        return "xlsx".into();
    }
    // crude text detection: printable
    if bytes
        .iter()
        .all(|b| *b == 9 || *b == 10 || *b == 13 || (32..=127).contains(b))
    {
        return "txt".into();
    }
    "unknown".into()
}

/// Converts a lopdf Object to a 64-bit floating point number.
///
/// # Arguments
///
/// * `obj` - A reference to a lopdf Object to convert
///
/// # Returns
///
/// The numeric value as f64:
/// - For `Object::Integer`: the integer value cast to f64
/// - For `Object::Real`: the real value cast to f64
/// - For all other object types: 0.0
///
/// # Use Case
///
/// This is commonly used when extracting numeric values from PDF objects,
/// such as page dimensions, font sizes, or coordinate values.
pub fn obj_to_f64(obj: &lopdf::Object) -> f64 {
    match obj {
        lopdf::Object::Integer(i) => *i as f64,
        lopdf::Object::Real(r) => *r as f64,
        _ => 0.0,
    }
}
