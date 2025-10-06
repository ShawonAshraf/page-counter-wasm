# PDF Page Counter - PDF.js Implementation

This is a JavaScript-based PDF page counter implementation using [PDF.js](https://mozilla.github.io/pdf.js/) library. It provides a simple API to count pages and extract detailed information from PDF files in the browser or Node.js.

## Features

- ✅ **Count PDF Pages** - Quickly count the number of pages in a PDF
- 📏 **Extract Page Dimensions** - Get detailed size information for each page
- 📐 **Multiple Units** - Page sizes in points, millimeters, and inches
- 🔄 **Orientation Detection** - Automatically detect portrait/landscape/square orientation
- 📋 **Paper Size Recognition** - Identify common paper sizes (A4, Letter, Legal, etc.)
- 🔍 **Uniformity Check** - Detect if all pages have the same dimensions
- ✨ **Easy to Use** - Simple, promise-based API

## Installation

**For direct browser usage (no build step):**
- ✅ No installation needed! `counter.js` uses PDF.js from CDN
- Just serve the files with any web server

**For bundler usage (webpack/vite/rollup):**
```bash
npm install
```
Then use `counter-local.js` instead of `counter.js`

## Usage

### In the Browser (No Build Step)

The default `counter.js` uses PDF.js from CDN, so it works directly in browsers without any build tools:

```html
<script type="module">
  import { countPdfPages, getPdfInfo } from './counter.js';

  // From file input
  const fileInput = document.querySelector('input[type="file"]');
  fileInput.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    const arrayBuffer = await file.arrayBuffer();
    
    // Count pages
    const pageCount = await countPdfPages(arrayBuffer);
    console.log(`PDF has ${pageCount} pages`);
    
    // Get detailed info
    const info = await getPdfInfo(arrayBuffer);
    console.log('Page details:', info);
  });
</script>
```

### With a Bundler (webpack/vite/rollup)

If you're using a bundler, use `counter-local.js` which imports from node_modules:

```html
<script type="module">
  import { countPdfPages, getPdfInfo } from './counter.js';

  // From file input
  const fileInput = document.querySelector('input[type="file"]');
  fileInput.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    const arrayBuffer = await file.arrayBuffer();
    
    // Count pages
    const pageCount = await countPdfPages(arrayBuffer);
    console.log(`PDF has ${pageCount} pages`);
    
    // Get detailed info
    const info = await getPdfInfo(arrayBuffer);
    console.log('Page details:', info);
  });
</script>
```

### API Reference

#### `countPdfPages(pdfData)`

Count the number of pages in a PDF.

**Parameters:**
- `pdfData` (ArrayBuffer | Uint8Array | string) - PDF file data

**Returns:** `Promise<number>` - Number of pages

**Example:**
```javascript
const pageCount = await countPdfPages(arrayBuffer);
console.log(`PDF has ${pageCount} pages`);
```

---

#### `getPdfInfo(pdfData)`

Get detailed information about all pages in a PDF.

**Parameters:**
- `pdfData` (ArrayBuffer | Uint8Array | string) - PDF file data

**Returns:** `Promise<Object>` - Object containing:
  - `pageCount` (number) - Total number of pages
  - `pages` (Array) - Array of page information objects
    - `pageNumber` (number) - Page number (1-indexed)
    - `width` (number) - Page width in points
    - `height` (number) - Page height in points
    - `widthMm` (number) - Page width in millimeters
    - `heightMm` (number) - Page height in millimeters
    - `widthInches` (number) - Page width in inches
    - `heightInches` (number) - Page height in inches
    - `orientation` (string) - Page orientation ('portrait', 'landscape', or 'square')

**Example:**
```javascript
const info = await getPdfInfo(arrayBuffer);
console.log(`PDF has ${info.pageCount} pages`);
info.pages.forEach(page => {
  console.log(`Page ${page.pageNumber}: ${page.widthMm} x ${page.heightMm} mm`);
});
```

---

#### `getPdfSummary(pdfData)`

Get a quick summary of the PDF file.

**Parameters:**
- `pdfData` (ArrayBuffer | Uint8Array | string) - PDF file data

**Returns:** `Promise<Object>` - Summary object containing:
  - `pageCount` (number) - Total number of pages
  - `hasUniformPages` (boolean) - Whether all pages have the same size
  - `commonSize` (string) - Common size if uniform (in mm)
  - `commonSizeInches` (string) - Common size if uniform (in inches)
  - `orientation` (string) - Orientation if uniform
  - `paperSize` (string) - Identified paper size (e.g., "A4", "Letter")
  - `uniqueSizes` (Array) - Array of unique sizes if not uniform

**Example:**
```javascript
const summary = await getPdfSummary(arrayBuffer);
console.log(summary);
// {
//   pageCount: 10,
//   hasUniformPages: true,
//   commonSize: "210.00 x 297.00 mm",
//   paperSize: "A4"
// }
```

---

#### `checkPdfUniformity(pdfData, tolerance = 0.5)`

Check if all pages have the same size.

**Parameters:**
- `pdfData` (ArrayBuffer | Uint8Array | string) - PDF file data
- `tolerance` (number) - Tolerance for size comparison in points (default: 0.5)

**Returns:** `Promise<Object>` - Object containing:
  - `isUniform` (boolean) - Whether all pages have the same size
  - `commonSize` (Object) - Common size if uniform
  - `uniqueSizes` (Array) - Array of unique page sizes found

**Example:**
```javascript
const uniformity = await checkPdfUniformity(arrayBuffer);
if (uniformity.isUniform) {
  console.log('All pages are the same size');
} else {
  console.log(`Found ${uniformity.uniqueSizes.length} different page sizes`);
}
```

---

#### `isValidPdf(pdfData)`

Validate if the provided data is a valid PDF.

**Parameters:**
- `pdfData` (ArrayBuffer | Uint8Array) - PDF file data

**Returns:** `Promise<boolean>` - True if valid PDF, false otherwise

**Example:**
```javascript
const isValid = await isValidPdf(arrayBuffer);
if (!isValid) {
  console.error('Invalid PDF file');
}
```

## Demo

A demo HTML file (`demo-pdfjs.html`) is included that shows how to use the PDF counter with a nice UI.

To run the demo:

1. Start a local web server (required for ES6 modules):
   ```bash
   npm run dev
   ```
   
   Or using Python:
   ```bash
   python3 -m http.server 8000
   ```

2. Open your browser and navigate to:
   ```
   http://localhost:8000/demo-pdfjs.html
   ```

3. Upload a PDF file and see the page count and detailed information!

## Supported Paper Sizes

The `getPdfSummary` function can automatically recognize these common paper sizes:

- **A-Series:** A3, A4, A5
- **US Letter Sizes:** Letter (8.5" × 11"), Legal (8.5" × 14")
- **Tabloid/Ledger:** Tabloid (11" × 17"), Ledger (17" × 11")

## Technical Details

### PDF.js Worker

The implementation automatically configures the PDF.js worker, which is required for PDF parsing. The worker is loaded from the `pdfjs-dist` package in `node_modules`.

### Browser Compatibility

This implementation uses modern JavaScript features:
- ES6 Modules
- Async/Await
- ArrayBuffer/Uint8Array

Make sure your target browsers support these features or use a transpiler like Babel.

### Memory Management

The implementation properly cleans up PDF.js resources by calling `destroy()` on PDF documents and `cleanup()` on pages to prevent memory leaks.

**ArrayBuffer Management**

The library automatically handles buffer copying internally to prevent ArrayBuffer detachment issues. You can safely call multiple functions with the same data:

```javascript
// ✅ Safe - buffer copying is handled internally
const arrayBuffer = await file.arrayBuffer();
const pdfData = new Uint8Array(arrayBuffer);

await countPdfPages(pdfData);      // Works
await getPdfInfo(pdfData);         // Works
await getPdfSummary(pdfData);      // Works

// All functions create internal copies as needed
```

## Differences from WASM Implementation

This project also includes a Rust/WASM implementation. Here are the key differences:

| Feature | PDF.js (JavaScript) | WASM (Rust) |
|---------|-------------------|-------------|
| **Language** | JavaScript | Rust |
| **Bundle Size** | Larger (~1.5MB) | Smaller (~200KB) |
| **Performance** | Good | Excellent |
| **Setup** | npm install | wasm-pack build |
| **Compatibility** | Modern browsers | Modern browsers |
| **Format Support** | PDF only | PDF, XLSX, TXT, MD |

Use the PDF.js implementation when:
- You only need PDF support
- You want simpler setup with npm
- You prefer pure JavaScript

Use the WASM implementation when:
- You need multi-format support
- You want smaller bundle size
- You need maximum performance

## Troubleshooting

### "Failed to initialize WASM" or worker errors

Make sure you're running the HTML files through a web server (not file:// protocol) because ES6 modules require HTTP/HTTPS.

### Module not found errors

Ensure that:
1. You've run `npm install`
2. The `node_modules` directory exists
3. You're using a web server that can serve from node_modules

### CORS errors

If you're loading PDFs from external URLs, make sure they have proper CORS headers set.

## Examples

### Simple Page Counter

```html
<!DOCTYPE html>
<html>
<head>
    <title>PDF Counter</title>
</head>
<body>
    <input type="file" id="pdfFile" accept=".pdf">
    <div id="result"></div>

    <script type="module">
        import { countPdfPages } from './counter.js';

        document.getElementById('pdfFile').addEventListener('change', async (e) => {
            const file = e.target.files[0];
            const arrayBuffer = await file.arrayBuffer();
            const count = await countPdfPages(arrayBuffer);
            document.getElementById('result').textContent = `Pages: ${count}`;
        });
    </script>
</body>
</html>
```

### Detailed Page Analysis

```javascript
import { getPdfInfo } from './counter.js';

async function analyzePdf(file) {
    const arrayBuffer = await file.arrayBuffer();
    const info = await getPdfInfo(arrayBuffer);
    
    console.log(`Total pages: ${info.pageCount}`);
    
    // Find the largest page
    const largest = info.pages.reduce((max, page) => 
        (page.width * page.height > max.width * max.height) ? page : max
    );
    
    console.log(`Largest page: #${largest.pageNumber} (${largest.widthMm}×${largest.heightMm}mm)`);
    
    // Find landscape pages
    const landscapes = info.pages.filter(p => p.orientation === 'landscape');
    console.log(`Landscape pages: ${landscapes.map(p => p.pageNumber).join(', ')}`);
}
```

## License

ISC

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Credits

This implementation uses [Mozilla's PDF.js](https://mozilla.github.io/pdf.js/) library for PDF parsing.
