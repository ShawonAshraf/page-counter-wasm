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