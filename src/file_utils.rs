pub fn mm_from_pt(pt: f64) -> f64 {
    // 1 point = 1/72 inch; 1 inch = 25.4 mm
    pt / 72.0 * 25.4
}

pub fn a4_mm() -> (f64, f64) {
    (210.0, 297.0)
}

pub fn letter_mm() -> (f64, f64) {
    (215.9, 279.4)
}

/// detect mime-like type from filename or magic bytes
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

pub fn obj_to_f64(obj: &lopdf::Object) -> f64 {
    match obj {
        lopdf::Object::Integer(i) => *i as f64,
        lopdf::Object::Real(r) => *r as f64,
        _ => 0.0,
    }
}
