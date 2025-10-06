//! Ultra-fast PDF page counter inspired by PDF.js
//!
//! This module replicates PDF.js's approach: search for /Type /Pages
//! and extract /Count without parsing the entire document.
//! Works directly on bytes without UTF-8 validation for maximum speed.

/// Count PDF pages using byte-level search (PDF.js approach)
/// 
/// This is the fastest possible approach - searches bytes directly
/// without UTF-8 validation or building any data structures.
pub fn count_pages_ultra_fast(bytes: &[u8]) -> Option<usize> {
    // Search for "/Type/Pages" or "/Type /Pages" patterns
    // The /Count value near this indicates total pages
    
    let mut max_count = 0;
    let mut found_pages = false;
    
    // Search for all occurrences of "Pages" in the document
    for i in 0..bytes.len().saturating_sub(20) {
        // Look for "/Pages" pattern
        if matches_pattern(bytes, i, b"/Pages") {
            // Found a Pages reference, look for /Count nearby
            // Search in the next 500 bytes
            let search_end = (i + 500).min(bytes.len());
            
            if let Some(count) = find_count_after_position(bytes, i, search_end) {
                found_pages = true;
                if count > max_count && count < 1_000_000 {
                    max_count = count;
                }
            }
        }
    }
    
    if found_pages && max_count > 0 {
        Some(max_count)
    } else {
        None
    }
}

/// Check if pattern matches at position
#[inline]
fn matches_pattern(bytes: &[u8], pos: usize, pattern: &[u8]) -> bool {
    if pos + pattern.len() > bytes.len() {
        return false;
    }
    
    for (i, &b) in pattern.iter().enumerate() {
        if bytes[pos + i] != b {
            return false;
        }
    }
    
    true
}

/// Find /Count value after a position
fn find_count_after_position(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    // Look for "/Count" pattern
    for i in start..end.saturating_sub(10) {
        if matches_pattern(bytes, i, b"/Count") {
            // Skip past "/Count" and whitespace
            let mut pos = i + 6; // length of "/Count"
            
            // Skip whitespace
            while pos < end && is_whitespace(bytes[pos]) {
                pos += 1;
            }
            
            // Extract digits
            let mut num = 0usize;
            let mut found_digit = false;
            
            while pos < end && bytes[pos].is_ascii_digit() {
                found_digit = true;
                num = num * 10 + (bytes[pos] - b'0') as usize;
                pos += 1;
                
                // Safety check
                if num > 1_000_000 {
                    return None;
                }
            }
            
            if found_digit {
                return Some(num);
            }
        }
    }
    
    None
}

/// Check if byte is PDF whitespace
#[inline]
fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\0')
}

/// Extract MediaBox dimensions (fast byte-level approach)
pub fn extract_mediabox_ultra_fast(bytes: &[u8]) -> Option<(f64, f64)> {
    // Search for first /MediaBox
    for i in 0..bytes.len().saturating_sub(50) {
        if matches_pattern(bytes, i, b"/MediaBox") {
            // Look for [ bracket after /MediaBox
            let mut pos = i + 9; // length of "/MediaBox"
            let search_end = (pos + 100).min(bytes.len());
            
            // Find opening bracket
            while pos < search_end && bytes[pos] != b'[' {
                pos += 1;
            }
            
            if pos >= search_end {
                continue;
            }
            
            pos += 1; // skip [
            
            // Extract 4 numbers: [x0 y0 x1 y1]
            let mut numbers = Vec::with_capacity(4);
            
            while numbers.len() < 4 && pos < search_end {
                // Skip whitespace
                while pos < search_end && is_whitespace(bytes[pos]) {
                    pos += 1;
                }
                
                // Check for closing bracket
                if bytes[pos] == b']' {
                    break;
                }
                
                // Parse number
                if let Some((num, new_pos)) = parse_float(bytes, pos, search_end) {
                    numbers.push(num);
                    pos = new_pos;
                } else {
                    break;
                }
            }
            
            if numbers.len() >= 4 {
                let width = (numbers[2] - numbers[0]).abs();
                let height = (numbers[3] - numbers[1]).abs();
                
                if width > 0.0 && width < 10000.0 && height > 0.0 && height < 10000.0 {
                    return Some((width, height));
                }
            }
        }
    }
    
    None
}

/// Parse a float from bytes
fn parse_float(bytes: &[u8], start: usize, end: usize) -> Option<(f64, usize)> {
    let mut pos = start;
    let mut num_str = Vec::new();
    
    // Handle negative sign
    if pos < end && bytes[pos] == b'-' {
        num_str.push(b'-');
        pos += 1;
    }
    
    // Parse digits and decimal point
    while pos < end {
        let b = bytes[pos];
        if b.is_ascii_digit() || b == b'.' {
            num_str.push(b);
            pos += 1;
        } else if is_whitespace(b) || b == b']' {
            break;
        } else {
            break;
        }
    }
    
    if num_str.is_empty() {
        return None;
    }
    
    // Convert to string and parse
    if let Ok(s) = std::str::from_utf8(&num_str) {
        if let Ok(num) = s.parse::<f64>() {
            return Some((num, pos));
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matches_pattern() {
        let data = b"hello /Pages world";
        assert!(matches_pattern(data, 6, b"/Pages"));
        assert!(!matches_pattern(data, 7, b"/Pages"));
    }
    
    #[test]
    fn test_is_whitespace() {
        assert!(is_whitespace(b' '));
        assert!(is_whitespace(b'\n'));
        assert!(!is_whitespace(b'a'));
    }
}

