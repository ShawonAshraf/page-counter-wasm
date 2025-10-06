/**
 * Node.js example for using the PDF counter
 * 
 * Run with: node example-node.mjs <path-to-pdf>
 */

import { readFile } from 'fs/promises';
import { countPdfPages, getPdfInfo, getPdfSummary } from './counter.js';

async function main() {
    // Get PDF path from command line arguments
    const pdfPath = process.argv[2];
    
    if (!pdfPath) {
        console.error('Usage: node example-node.mjs <path-to-pdf>');
        process.exit(1);
    }
    
    try {
        console.log(`\n📄 Analyzing PDF: ${pdfPath}\n`);
        
        // Read the PDF file as Uint8Array
        // Note: Functions internally handle buffer copying
        const fileBuffer = await readFile(pdfPath);
        const pdfData = new Uint8Array(fileBuffer);
        
        // Count pages
        console.log('⏳ Counting pages...');
        const pageCount = await countPdfPages(pdfData);
        console.log(`✅ Total pages: ${pageCount}\n`);
        
        // Get summary
        console.log('⏳ Getting summary...');
        const summary = await getPdfSummary(pdfData);
        console.log('✅ Summary:');
        console.log(`   Pages: ${summary.pageCount}`);
        console.log(`   Uniform pages: ${summary.hasUniformPages ? 'Yes' : 'No'}`);
        
        if (summary.hasUniformPages) {
            console.log(`   Common size: ${summary.commonSize}`);
            console.log(`   Orientation: ${summary.orientation}`);
            if (summary.paperSize) {
                console.log(`   Paper size: ${summary.paperSize}`);
            }
        } else {
            console.log(`   Unique sizes: ${summary.uniqueSizes.length}`);
            summary.uniqueSizes.forEach((size, idx) => {
                console.log(`   Size ${idx + 1}: ${size.dimensions} (${size.orientation}) - ${size.pageCount} page(s)`);
            });
        }
        
        console.log('');
        
        // Get detailed info (only for small PDFs)
        if (pageCount <= 20) {
            console.log('⏳ Getting detailed page info...');
            const info = await getPdfInfo(pdfData);
            console.log('✅ Page details:');
            info.pages.forEach(page => {
                console.log(`   Page ${page.pageNumber}: ${page.widthMm.toFixed(2)} × ${page.heightMm.toFixed(2)} mm (${page.orientation})`);
            });
        } else {
            console.log(`ℹ️  PDF has too many pages (${pageCount}) to show detailed info`);
        }
        
        console.log('\n✨ Analysis complete!\n');
        
    } catch (error) {
        console.error(`\n❌ Error: ${error.message}\n`);
        process.exit(1);
    }
}

main();
