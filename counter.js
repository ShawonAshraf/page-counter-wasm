/**
 * PDF Page Counter using PDF.js
 * 
 * This module provides functionality to count pages in PDF files
 * and extract page size information using the pdf.js library.
 */

// Import PDF.js from CDN for browser compatibility
const pdfjsLib = await import('https://cdn.jsdelivr.net/npm/pdfjs-dist@4.4.168/+esm');

// Configure the worker - important for PDF.js to work properly
pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.4.168/build/pdf.worker.min.mjs';

/**
 * Counts the number of pages in a PDF file
 * 
 * @param {ArrayBuffer|Uint8Array|string} pdfData - PDF file data (ArrayBuffer, Uint8Array, or file path)
 * @returns {Promise<number>} Number of pages in the PDF
 * @throws {Error} If the PDF cannot be loaded or parsed
 * 
 * @example
 * // From a file input
 * const file = event.target.files[0];
 * const arrayBuffer = await file.arrayBuffer();
 * const pageCount = await countPdfPages(arrayBuffer);
 * console.log(`PDF has ${pageCount} pages`);
 */
export async function countPdfPages(pdfData) {
    try {
        // Create a copy to prevent buffer detachment issues
        const dataCopy = pdfData instanceof Uint8Array ? pdfData.slice() : new Uint8Array(pdfData).slice();
        const loadingTask = pdfjsLib.getDocument(dataCopy);
        const pdf = await loadingTask.promise;
        const pageCount = pdf.numPages;
        
        // Clean up
        await pdf.destroy();
        
        return pageCount;
    } catch (error) {
        throw new Error(`Failed to count PDF pages: ${error.message}`);
    }
}

/**
 * Gets detailed information about all pages in a PDF
 * 
 * @param {ArrayBuffer|Uint8Array|string} pdfData - PDF file data
 * @returns {Promise<Object>} Object containing page count and page sizes
 * @returns {number} return.pageCount - Total number of pages
 * @returns {Array<Object>} return.pages - Array of page information objects
 * @returns {number} return.pages[].pageNumber - Page number (1-indexed)
 * @returns {number} return.pages[].width - Page width in points
 * @returns {number} return.pages[].height - Page height in points
 * @returns {number} return.pages[].widthMm - Page width in millimeters
 * @returns {number} return.pages[].heightMm - Page height in millimeters
 * @returns {number} return.pages[].widthInches - Page width in inches
 * @returns {number} return.pages[].heightInches - Page height in inches
 * @returns {string} return.pages[].orientation - Page orientation ('portrait', 'landscape', or 'square')
 * 
 * @example
 * const file = event.target.files[0];
 * const arrayBuffer = await file.arrayBuffer();
 * const info = await getPdfInfo(arrayBuffer);
 * console.log(`PDF has ${info.pageCount} pages`);
 * info.pages.forEach(page => {
 *   console.log(`Page ${page.pageNumber}: ${page.widthMm} x ${page.heightMm} mm`);
 * });
 */
export async function getPdfInfo(pdfData) {
    try {
        // Create a copy to prevent buffer detachment issues
        const dataCopy = pdfData instanceof Uint8Array ? pdfData.slice() : new Uint8Array(pdfData).slice();
        const loadingTask = pdfjsLib.getDocument(dataCopy);
        const pdf = await loadingTask.promise;
        const pageCount = pdf.numPages;
        
        const pages = [];
        
        // Get information for each page
        for (let i = 1; i <= pageCount; i++) {
            const page = await pdf.getPage(i);
            const viewport = page.getViewport({ scale: 1.0 });
            
            // PDF uses points (1 point = 1/72 inch)
            const widthPoints = viewport.width;
            const heightPoints = viewport.height;
            
            // Convert to other units
            const widthMm = (widthPoints / 72) * 25.4;
            const heightMm = (heightPoints / 72) * 25.4;
            const widthInches = widthPoints / 72;
            const heightInches = heightPoints / 72;
            
            // Determine orientation
            let orientation = 'portrait';
            if (widthPoints > heightPoints) {
                orientation = 'landscape';
            } else if (Math.abs(widthPoints - heightPoints) < 1) {
                orientation = 'square';
            }
            
            pages.push({
                pageNumber: i,
                width: Math.round(widthPoints * 100) / 100,
                height: Math.round(heightPoints * 100) / 100,
                widthMm: Math.round(widthMm * 100) / 100,
                heightMm: Math.round(heightMm * 100) / 100,
                widthInches: Math.round(widthInches * 100) / 100,
                heightInches: Math.round(heightInches * 100) / 100,
                orientation
            });
            
            // Clean up the page
            page.cleanup();
        }
        
        // Clean up the document
        await pdf.destroy();
        
        return {
            pageCount,
            pages
        };
    } catch (error) {
        throw new Error(`Failed to get PDF info: ${error.message}`);
    }
}

/**
 * Checks if all pages in a PDF have the same size
 * 
 * @param {ArrayBuffer|Uint8Array|string} pdfData - PDF file data
 * @param {number} tolerance - Tolerance for size comparison in points (default: 0.5)
 * @returns {Promise<Object>} Object containing uniformity information
 * @returns {boolean} return.isUniform - Whether all pages have the same size
 * @returns {Object} return.commonSize - Common size if uniform (in various units)
 * @returns {Array<Object>} return.uniqueSizes - Array of unique page sizes found
 * 
 * @example
 * const uniformity = await checkPdfUniformity(arrayBuffer);
 * if (uniformity.isUniform) {
 *   console.log('All pages are the same size:', uniformity.commonSize);
 * } else {
 *   console.log('Pages have different sizes:', uniformity.uniqueSizes);
 * }
 */
export async function checkPdfUniformity(pdfData, tolerance = 0.5) {
    try {
        const info = await getPdfInfo(pdfData);
        
        if (info.pages.length === 0) {
            return {
                isUniform: true,
                commonSize: null,
                uniqueSizes: []
            };
        }
        
        const uniqueSizes = [];
        
        for (const page of info.pages) {
            const isDuplicate = uniqueSizes.some(size => 
                Math.abs(size.width - page.width) < tolerance &&
                Math.abs(size.height - page.height) < tolerance
            );
            
            if (!isDuplicate) {
                uniqueSizes.push({
                    width: page.width,
                    height: page.height,
                    widthMm: page.widthMm,
                    heightMm: page.heightMm,
                    widthInches: page.widthInches,
                    heightInches: page.heightInches,
                    orientation: page.orientation,
                    pageNumbers: [page.pageNumber]
                });
            } else {
                const existingSize = uniqueSizes.find(size =>
                    Math.abs(size.width - page.width) < tolerance &&
                    Math.abs(size.height - page.height) < tolerance
                );
                existingSize.pageNumbers.push(page.pageNumber);
            }
        }
        
        return {
            isUniform: uniqueSizes.length === 1,
            commonSize: uniqueSizes.length === 1 ? uniqueSizes[0] : null,
            uniqueSizes
        };
    } catch (error) {
        throw new Error(`Failed to check PDF uniformity: ${error.message}`);
    }
}

/**
 * Gets a quick summary of a PDF file
 * 
 * @param {ArrayBuffer|Uint8Array|string} pdfData - PDF file data
 * @returns {Promise<Object>} Summary object with page count and size information
 * 
 * @example
 * const summary = await getPdfSummary(arrayBuffer);
 * console.log(summary);
 * // {
 * //   pageCount: 10,
 * //   hasUniformPages: true,
 * //   commonSize: "210.00 x 297.00 mm (A4, Portrait)"
 * // }
 */
export async function getPdfSummary(pdfData) {
    try {
        // Get info once and process it locally to avoid multiple buffer operations
        const info = await getPdfInfo(pdfData);
        
        // Check uniformity locally using the info we already have
        const tolerance = 0.5;
        const uniqueSizes = [];
        
        for (const page of info.pages) {
            const isDuplicate = uniqueSizes.some(size => 
                Math.abs(size.width - page.width) < tolerance &&
                Math.abs(size.height - page.height) < tolerance
            );
            
            if (!isDuplicate) {
                uniqueSizes.push({
                    width: page.width,
                    height: page.height,
                    widthMm: page.widthMm,
                    heightMm: page.heightMm,
                    widthInches: page.widthInches,
                    heightInches: page.heightInches,
                    orientation: page.orientation,
                    pageNumbers: [page.pageNumber]
                });
            } else {
                const existingSize = uniqueSizes.find(size =>
                    Math.abs(size.width - page.width) < tolerance &&
                    Math.abs(size.height - page.height) < tolerance
                );
                existingSize.pageNumbers.push(page.pageNumber);
            }
        }
        
        const isUniform = uniqueSizes.length === 1;
        
        let summary = {
            pageCount: info.pageCount,
            hasUniformPages: isUniform
        };
        
        if (isUniform && uniqueSizes[0]) {
            const size = uniqueSizes[0];
            summary.commonSize = `${size.widthMm.toFixed(2)} x ${size.heightMm.toFixed(2)} mm`;
            summary.commonSizeInches = `${size.widthInches.toFixed(2)} x ${size.heightInches.toFixed(2)} inches`;
            summary.orientation = size.orientation;
            
            // Try to identify common paper sizes
            summary.paperSize = identifyPaperSize(size.widthMm, size.heightMm);
        } else {
            summary.uniqueSizes = uniqueSizes.map(size => ({
                dimensions: `${size.widthMm.toFixed(2)} x ${size.heightMm.toFixed(2)} mm`,
                orientation: size.orientation,
                pageCount: size.pageNumbers.length,
                pageNumbers: size.pageNumbers
            }));
        }
        
        return summary;
    } catch (error) {
        throw new Error(`Failed to get PDF summary: ${error.message}`);
    }
}

/**
 * Identifies common paper sizes based on dimensions
 * 
 * @private
 * @param {number} widthMm - Width in millimeters
 * @param {number} heightMm - Height in millimeters
 * @returns {string|null} Paper size name or null if not recognized
 */
function identifyPaperSize(widthMm, heightMm) {
    const tolerance = 5; // mm
    
    const paperSizes = {
        'A4': { width: 210, height: 297 },
        'A3': { width: 297, height: 420 },
        'A5': { width: 148, height: 210 },
        'Letter': { width: 215.9, height: 279.4 },
        'Legal': { width: 215.9, height: 355.6 },
        'Tabloid': { width: 279.4, height: 431.8 },
        'Ledger': { width: 431.8, height: 279.4 }
    };
    
    for (const [name, size] of Object.entries(paperSizes)) {
        // Check both orientations
        if (
            (Math.abs(widthMm - size.width) < tolerance && Math.abs(heightMm - size.height) < tolerance) ||
            (Math.abs(widthMm - size.height) < tolerance && Math.abs(heightMm - size.width) < tolerance)
        ) {
            return name;
        }
    }
    
    return null;
}

/**
 * Validates if the provided data is a valid PDF
 * 
 * @param {ArrayBuffer|Uint8Array} pdfData - PDF file data
 * @returns {Promise<boolean>} True if valid PDF, false otherwise
 * 
 * @example
 * const isValid = await isValidPdf(arrayBuffer);
 * if (!isValid) {
 *   console.error('Invalid PDF file');
 * }
 */
export async function isValidPdf(pdfData) {
    try {
        // Create a copy to prevent buffer detachment issues
        const dataCopy = pdfData instanceof Uint8Array ? pdfData.slice() : new Uint8Array(pdfData).slice();
        const loadingTask = pdfjsLib.getDocument(dataCopy);
        const pdf = await loadingTask.promise;
        await pdf.destroy();
        return true;
    } catch (error) {
        return false;
    }
}

// Export the pdfjsLib for advanced usage
export { pdfjsLib };

// Default export with all functions
export default {
    countPdfPages,
    getPdfInfo,
    checkPdfUniformity,
    getPdfSummary,
    isValidPdf,
    pdfjsLib
};
