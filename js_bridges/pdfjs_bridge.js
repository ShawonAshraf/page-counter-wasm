/**
 * Bridge between Rust WASM and PDF.js
 * This file is imported by Rust using wasm-bindgen
 */

/**
 * Count PDF pages using PDF.js
 * @param {Uint8Array} bytes - The PDF file bytes
 * @returns {Promise<string>} JSON string with page count and dimensions
 */
export async function count_pdf_pages_js(bytes) {
    try {
        // Check if PDF.js is available
        if (typeof pdfjsLib === 'undefined') {
            throw new Error('PDF.js not loaded');
        }

        // Load the PDF document
        const loadingTask = pdfjsLib.getDocument({data: bytes});
        const pdf = await loadingTask.promise;
        
        const pageCount = pdf.numPages;
        
        // Get dimensions from first page
        let widthPt = 595.0;  // A4 default
        let heightPt = 842.0;
        
        try {
            const firstPage = await pdf.getPage(1);
            const viewport = firstPage.getViewport({scale: 1.0});
            widthPt = viewport.width;
            heightPt = viewport.height;
        } catch (e) {
            console.warn('Could not get page dimensions, using defaults');
        }
        
        // Return as JSON string
        return JSON.stringify({
            page_count: pageCount,
            width_pt: widthPt,
            height_pt: heightPt
        });
        
    } catch (error) {
        console.error('PDF.js error:', error);
        throw error;
    }
}

