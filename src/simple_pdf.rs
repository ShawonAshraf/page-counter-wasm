//! Simple, robust PDF page counter using regex patterns
//!
//! This module provides a lightweight PDF page counter that uses
//! simple regex patterns to find the page count in the PDF structure.
//! It's designed to be fast and panic-free for WebAssembly.

/// Extract page count from PDF using simple pattern matching
/// 
/// This function looks for the /Count value in the PDF's Pages dictionary
/// using a conservative approach that avoids panics.
pub fn count_pdf_pages_simple(bytes: &[u8]) -> Option<usize> {
    // Convert to string - we only need ASCII for PDF structure
    let content = std::str::from_utf8(bytes).ok()?;
    
    // Strategy: Find "/Type/Pages" or "/Type /Pages" followed by "/Count"
    // This is the most common pattern in PDF files
    
    // Find all positions where we see /Count followed by a number
    let mut best_count: Option<usize> = None;
    let mut highest_count = 0;
    
    // Search for /Count patterns throughout the document
    for (i, _) in content.match_indices("/Count") {
        // Get a safe slice after /Count
        let start = i + 6; // length of "/Count"
        if start >= content.len() {
            continue;
        }
        
        // Look at the next 50 characters max
        let end = (start + 50).min(content.len());
        let after_count = &content[start..end];
        
        // Extract the number after /Count
        if let Some(num) = extract_number(after_count) {
            // The highest /Count value is usually the total page count
            // (smaller counts are for subsections)
            if num > highest_count && num < 1_000_000 {
                highest_count = num;
                best_count = Some(num);
            }
        }
    }
    
    best_count
}

/// Extract the first number from a string
fn extract_number(s: &str) -> Option<usize> {
    // Skip whitespace and find digits
    let trimmed = s.trim_start();
    
    // Collect consecutive digits
    let num_str: String = trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    
    if num_str.is_empty() {
        return None;
    }
    
    num_str.parse().ok()
}

/// Extract MediaBox dimensions from PDF
pub fn extract_mediabox_simple(bytes: &[u8]) -> Option<(f64, f64)> {
    let content = std::str::from_utf8(bytes).ok()?;
    
    // Find first /MediaBox
    let mediabox_idx = content.find("/MediaBox")?;
    let after_mediabox = &content[mediabox_idx..];
    
    // Find the array bounds
    let start_idx = after_mediabox.find('[')?;
    let end_idx = after_mediabox.find(']')?;
    
    if end_idx <= start_idx || end_idx > after_mediabox.len() {
        return None;
    }
    
    // Get array content
    let array_str = &after_mediabox[start_idx + 1..end_idx];
    
    // Parse numbers from the array
    let numbers: Vec<f64> = array_str
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    
    if numbers.len() >= 4 {
        let width = (numbers[2] - numbers[0]).abs();
        let height = (numbers[3] - numbers[1]).abs();
        
        // Sanity check
        if width > 0.0 && width < 10000.0 && height > 0.0 && height < 10000.0 {
            return Some((width, height));
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("  42"), Some(42));
        assert_eq!(extract_number("123 /Kids"), Some(123));
        assert_eq!(extract_number("  "), None);
    }
}

