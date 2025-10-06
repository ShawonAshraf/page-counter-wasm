//! Ultra-fast PDF page counter inspired by PDF.js
//!
//! This module replicates PDF.js's approach: search for /Type /Pages
//! and extract /Count without parsing the entire document.
//! Works directly on bytes without UTF-8 validation for maximum speed.

/// Count PDF pages using byte-level search (PDF.js approach)
/// 
/// This follows the PDF specification: read from end of file,
/// find startxref, follow trailer to Root to Pages to /Count
pub fn count_pages_ultra_fast(bytes: &[u8]) -> Option<usize> {
    // CORRECT APPROACH: Start from the END of the file (PDF spec)
    // 1. Find "startxref" near end of file
    // 2. Get the xref offset
    // 3. Read trailer dictionary
    // 4. Follow Root -> Pages -> /Count
    if let Some(count) = parse_from_end_of_file(bytes) {
        return Some(count);
    }
    
    // FALLBACK 1: Search for /Type/Pages or /Type /Pages with /Count nearby
    if let Some(count) = search_type_pages_pattern(bytes) {
        return Some(count);
    }
    
    // FALLBACK 2: Search for any /Count values and take the maximum
    if let Some(count) = search_all_count_values(bytes) {
        return Some(count);
    }
    
    // FALLBACK 3: Search with expanded window
    if let Some(count) = search_pages_with_large_window(bytes) {
        return Some(count);
    }
    
    // FALLBACK 4: Ultra-aggressive - search for "Count" (without /)
    if let Some(count) = search_count_without_slash(bytes) {
        return Some(count);
    }
    
    None
}

/// Parse PDF from end of file following PDF specification
/// This is how PDF.js actually works
fn parse_from_end_of_file(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 100 {
        return None;
    }
    
    // PDF files end with "%%EOF", search backwards from end
    // startxref should be within last 1024 bytes
    let search_start = bytes.len().saturating_sub(1024);
    let end_section = &bytes[search_start..];
    
    // Find "startxref" - it points to the xref table offset
    let startxref_pattern = b"startxref";
    let mut startxref_pos = None;
    
    for i in 0..end_section.len().saturating_sub(startxref_pattern.len()) {
        if matches_pattern(end_section, i, startxref_pattern) {
            startxref_pos = Some(search_start + i);
        }
    }
    
    let startxref_pos = startxref_pos?;
    
    // Extract the offset number after "startxref"
    let after_startxref = &bytes[startxref_pos + 9..]; // 9 = len("startxref")
    let xref_offset = extract_first_number(after_startxref)?;
    
    if xref_offset >= bytes.len() {
        return None;
    }
    
    // Read from xref_offset to find trailer
    let from_xref = &bytes[xref_offset..];
    
    // Find "trailer" keyword
    let trailer_pattern = b"trailer";
    let mut trailer_pos = None;
    
    for i in 0..from_xref.len().saturating_sub(trailer_pattern.len()).min(5000) {
        if matches_pattern(from_xref, i, trailer_pattern) {
            trailer_pos = Some(i);
            break;
        }
    }
    
    let trailer_pos = trailer_pos?;
    let trailer_section = &from_xref[trailer_pos..trailer_pos.saturating_add(2000).min(from_xref.len())];
    
    // In trailer, find /Root reference
    let root_obj_id = find_object_reference(trailer_section, b"/Root")?;
    
    // Find the Root object in the document
    let root_section = find_object_by_id(bytes, root_obj_id)?;
    
    // In Root, find /Pages reference  
    let pages_obj_id = find_object_reference(root_section, b"/Pages")?;
    
    // Find the Pages object
    let pages_section = find_object_by_id(bytes, pages_obj_id)?;
    
    // In Pages, find /Count value
    find_count_value(pages_section)
}

/// Extract first number from bytes
fn extract_first_number(bytes: &[u8]) -> Option<usize> {
    let mut pos = 0;
    
    // Skip whitespace
    while pos < bytes.len() && is_whitespace(bytes[pos]) {
        pos += 1;
    }
    
    // Extract digits
    let mut num = 0usize;
    let mut found_digit = false;
    
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        found_digit = true;
        num = num * 10 + (bytes[pos] - b'0') as usize;
        pos += 1;
        
        if num > 100_000_000 {
            return None;
        }
    }
    
    if found_digit {
        Some(num)
    } else {
        None
    }
}

/// Find object reference after a keyword like /Root or /Pages
fn find_object_reference(bytes: &[u8], keyword: &[u8]) -> Option<usize> {
    for i in 0..bytes.len().saturating_sub(keyword.len() + 10) {
        if matches_pattern(bytes, i, keyword) {
            // After keyword, extract the object ID (number before "0 R")
            let after = &bytes[i + keyword.len()..];
            return extract_first_number(after);
        }
    }
    None
}

/// Find object by ID - search for "ID 0 obj" pattern
fn find_object_by_id(bytes: &[u8], obj_id: usize) -> Option<&[u8]> {
    let pattern = format!("{} 0 obj", obj_id);
    let pattern_bytes = pattern.as_bytes();
    
    for i in 0..bytes.len().saturating_sub(pattern_bytes.len()) {
        if matches_pattern(bytes, i, pattern_bytes) {
            // Return section from here to next "endobj" or 2000 bytes
            let start = i;
            let end = (i + 2000).min(bytes.len());
            return Some(&bytes[start..end]);
        }
    }
    None
}

/// Find /Count value in a section
fn find_count_value(bytes: &[u8]) -> Option<usize> {
    for i in 0..bytes.len().saturating_sub(10) {
        if matches_pattern(bytes, i, b"/Count") {
            let after = &bytes[i + 6..]; // 6 = len("/Count")
            return extract_first_number(after);
        }
    }
    None
}

/// Search for "Count" without slash (some PDFs might have unusual formatting)
fn search_count_without_slash(bytes: &[u8]) -> Option<usize> {
    let mut counts: Vec<usize> = Vec::new();
    
    for i in 0..bytes.len().saturating_sub(20) {
        // Look for "Count" pattern preceded by whitespace or /
        if i > 0 && (is_whitespace(bytes[i-1]) || bytes[i-1] == b'/') && matches_pattern(bytes, i, b"Count") {
            let mut pos = i + 5; // length of "Count"
            
            // Skip whitespace and special characters
            while pos < bytes.len() && (is_whitespace(bytes[pos]) || matches!(bytes[pos], b'(' | b')' | b'<' | b'>' | b'[' | b']' | b':')) {
                pos += 1;
            }
            
            // Extract digits
            let mut num = 0usize;
            let mut found_digit = false;
            
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                found_digit = true;
                num = num * 10 + (bytes[pos] - b'0') as usize;
                pos += 1;
                
                if num > 1_000_000 {
                    break;
                }
            }
            
            if found_digit && num > 0 && num < 1_000_000 {
                counts.push(num);
            }
        }
    }
    
    if counts.is_empty() {
        return None;
    }
    
    // Return the maximum
    let max_count = *counts.iter().max()?;
    
    if max_count > 0 {
        Some(max_count)
    } else {
        None
    }
}

/// Search for /Pages with a very large search window
/// Some PDFs have /Count far away from /Pages
fn search_pages_with_large_window(bytes: &[u8]) -> Option<usize> {
    let mut max_count = 0;
    let mut found_pages = false;
    
    // Search for all occurrences of "/Pages" in the document
    for i in 0..bytes.len().saturating_sub(50) {
        // Look for "/Pages" pattern
        if matches_pattern(bytes, i, b"/Pages") {
            // Search backward AND forward from /Pages
            // Some PDFs have /Count before /Pages in the same object
            let search_start = i.saturating_sub(2000);
            let search_end = (i + 5000).min(bytes.len());
            
            // Search backward first
            if let Some(count) = find_count_in_range(bytes, search_start, i) {
                found_pages = true;
                if count > max_count && count < 1_000_000 {
                    max_count = count;
                }
            }
            
            // Then search forward
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

/// Search for /Count in a specific range
fn find_count_in_range(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    for i in start..end.saturating_sub(10) {
        if matches_pattern(bytes, i, b"/Count") {
            let mut pos = i + 6; // length of "/Count"
            
            // Skip whitespace and special characters
            while pos < bytes.len() && (is_whitespace(bytes[pos]) || matches!(bytes[pos], b'(' | b')' | b'<' | b'>' | b'[' | b']' | b':')) {
                pos += 1;
            }
            
            // Extract digits
            let mut num = 0usize;
            let mut found_digit = false;
            
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                found_digit = true;
                num = num * 10 + (bytes[pos] - b'0') as usize;
                pos += 1;
                
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

/// Search for /Type/Pages patterns with associated /Count
fn search_type_pages_pattern(bytes: &[u8]) -> Option<usize> {
    let mut max_count = 0;
    let mut found_pages = false;
    
    // Search for all occurrences of "/Pages" in the document
    for i in 0..bytes.len().saturating_sub(20) {
        // Look for "/Pages" pattern
        if matches_pattern(bytes, i, b"/Pages") {
            // Found a Pages reference, look for /Count nearby
            // Search in the next 2000 bytes (increased from 500)
            let search_end = (i + 2000).min(bytes.len());
            
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

/// Search for all /Count values and return the maximum
/// This is more aggressive but works for most PDFs
fn search_all_count_values(bytes: &[u8]) -> Option<usize> {
    let mut counts: Vec<usize> = Vec::new();
    
    // Search for all occurrences of "/Count" in the document
    for i in 0..bytes.len().saturating_sub(20) {
        if matches_pattern(bytes, i, b"/Count") {
            // Extract the number after /Count
            let mut pos = i + 6; // length of "/Count"
            
            // Skip whitespace and special chars like ( ) < > [ ]
            while pos < bytes.len() && (is_whitespace(bytes[pos]) || matches!(bytes[pos], b'(' | b')' | b'<' | b'>' | b'[' | b']')) {
                pos += 1;
            }
            
            // Extract digits
            let mut num = 0usize;
            let mut found_digit = false;
            
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                found_digit = true;
                num = num * 10 + (bytes[pos] - b'0') as usize;
                pos += 1;
                
                if num > 1_000_000 {
                    break;
                }
            }
            
            if found_digit && num > 0 && num < 1_000_000 {
                counts.push(num);
            }
        }
    }
    
    if counts.is_empty() {
        return None;
    }
    
    // Return the maximum count found
    // In PDFs, the root Pages object always has the highest /Count
    let max_count = *counts.iter().max()?;
    
    if max_count > 0 {
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
            // Skip past "/Count" and whitespace/special chars
            let mut pos = i + 6; // length of "/Count"
            
            // Skip whitespace and special characters
            while pos < end && (is_whitespace(bytes[pos]) || matches!(bytes[pos], b'(' | b')' | b'<' | b'>' | b'[' | b']' | b':')) {
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

