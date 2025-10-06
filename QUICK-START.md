# Quick Start Guide - PDF.js Counter

## 🚀 Getting Started (3 steps)

### 1. Install Dependencies
```bash
npm install
```

### 2. Start a Web Server
```bash
npm run dev
```

### 3. Open the Demo
Navigate to `http://localhost:8000/demo-pdfjs.html` in your browser and upload a PDF!

---

## 📝 Common Use Cases

### Use Case 1: Simple Page Counter

```html
<input type="file" id="pdf" accept=".pdf">
<div id="count"></div>

<script type="module">
import { countPdfPages } from './counter.js';

document.getElementById('pdf').addEventListener('change', async (e) => {
    const buffer = await e.target.files[0].arrayBuffer();
    const count = await countPdfPages(buffer);
    document.getElementById('count').textContent = `${count} pages`;
});
</script>
```

### Use Case 2: Get Page Dimensions

```javascript
import { getPdfInfo } from './counter.js';

const info = await getPdfInfo(arrayBuffer);
console.log(`Total: ${info.pageCount} pages`);
info.pages.forEach(page => {
    console.log(`Page ${page.pageNumber}: ${page.widthMm}x${page.heightMm}mm`);
});
```

### Use Case 3: Quick Summary

```javascript
import { getPdfSummary } from './counter.js';

const summary = await getPdfSummary(arrayBuffer);
console.log(`Pages: ${summary.pageCount}`);
console.log(`Size: ${summary.commonSize}`);
console.log(`Paper: ${summary.paperSize}`);  // e.g., "A4", "Letter"
```

### Use Case 4: Check if All Pages Same Size

```javascript
import { checkPdfUniformity } from './counter.js';

const result = await checkPdfUniformity(arrayBuffer);
if (result.isUniform) {
    console.log('All pages are the same size!');
} else {
    console.log(`Found ${result.uniqueSizes.length} different sizes`);
}
```

### Use Case 5: Validate PDF

```javascript
import { isValidPdf } from './counter.js';

const isValid = await isValidPdf(arrayBuffer);
if (!isValid) {
    alert('Please upload a valid PDF file');
}
```

---

## 🎯 Real-World Examples

### Form with Validation

```javascript
import { countPdfPages, isValidPdf } from './counter.js';

async function handlePdfUpload(file) {
    // Validate
    const buffer = await file.arrayBuffer();
    if (!await isValidPdf(buffer)) {
        throw new Error('Invalid PDF');
    }
    
    // Count pages
    const pages = await countPdfPages(buffer);
    
    // Business logic
    if (pages > 100) {
        throw new Error('PDF must be 100 pages or less');
    }
    
    return { file, pages };
}
```

### Progress Indicator

```javascript
import { getPdfInfo } from './counter.js';

async function analyzePdfWithProgress(arrayBuffer, onProgress) {
    onProgress('Loading PDF...');
    const info = await getPdfInfo(arrayBuffer);
    
    onProgress(`Analyzing ${info.pageCount} pages...`);
    // Do something with pages...
    
    onProgress('Complete!');
    return info;
}
```

### Batch Processing

```javascript
import { countPdfPages } from './counter.js';

async function processManyPdfs(files) {
    const results = [];
    
    for (const file of files) {
        const buffer = await file.arrayBuffer();
        const pages = await countPdfPages(buffer);
        results.push({ 
            name: file.name, 
            pages 
        });
    }
    
    return results;
}
```

---

## 🔧 Integration Examples

### React Component

```jsx
import { useState } from 'react';
import { countPdfPages } from './counter.js';

function PdfCounter() {
    const [count, setCount] = useState(null);
    
    const handleFile = async (e) => {
        const file = e.target.files[0];
        const buffer = await file.arrayBuffer();
        const pages = await countPdfPages(buffer);
        setCount(pages);
    };
    
    return (
        <div>
            <input type="file" onChange={handleFile} accept=".pdf" />
            {count && <p>Pages: {count}</p>}
        </div>
    );
}
```

### Vue Component

```vue
<template>
  <div>
    <input type="file" @change="handleFile" accept=".pdf">
    <p v-if="pageCount">Pages: {{ pageCount }}</p>
  </div>
</template>

<script>
import { countPdfPages } from './counter.js';

export default {
  data() {
    return { pageCount: null };
  },
  methods: {
    async handleFile(e) {
      const file = e.target.files[0];
      const buffer = await file.arrayBuffer();
      this.pageCount = await countPdfPages(buffer);
    }
  }
}
</script>
```

### Node.js Script

```javascript
import { readFile } from 'fs/promises';
import { countPdfPages } from './counter.js';

const buffer = await readFile('document.pdf');
const pages = await countPdfPages(buffer);
console.log(`Document has ${pages} pages`);
```

---

## ⚡ Performance Tips

1. **Reuse buffers**: Don't re-read files if you need multiple operations
   ```javascript
   const buffer = await file.arrayBuffer();
   const count = await countPdfPages(buffer);
   const info = await getPdfInfo(buffer);  // Reuse buffer
   ```

2. **Use appropriate function**: Don't use `getPdfInfo()` if you only need count
   ```javascript
   // ❌ Slower - gets all page details
   const info = await getPdfInfo(buffer);
   const count = info.pageCount;
   
   // ✅ Faster - only counts pages
   const count = await countPdfPages(buffer);
   ```

3. **Handle large PDFs**: Show progress for large files
   ```javascript
   if (file.size > 10_000_000) {  // 10MB+
       showProgressBar();
   }
   const count = await countPdfPages(buffer);
   ```

---

## 🐛 Troubleshooting

### Problem: "Cannot perform Construct on a detached ArrayBuffer"
**Solution**: This issue has been fixed! The functions now handle buffer copying internally. You can safely call multiple functions with the same data:
```javascript
const arrayBuffer = await file.arrayBuffer();
const pdfData = new Uint8Array(arrayBuffer);

// Safe to call multiple times - buffer copying handled internally
const count = await countPdfPages(pdfData);
const info = await getPdfInfo(pdfData);
const summary = await getPdfSummary(pdfData);
```

### Problem: "Cannot use import statement outside a module"
**Solution**: Add `type="module"` to your script tag or use `.mjs` extension

### Problem: "Failed to fetch dynamically imported module"
**Solution**: Serve files through a web server, not `file://` protocol

### Problem: Worker errors
**Solution**: Ensure `node_modules/pdfjs-dist` is accessible from your web server

### Problem: Large bundle size
**Solution**: Consider the WASM implementation (200KB vs 1.5MB)

---

## 📚 Learn More

- Full API documentation: [README-PDFJS.md](README-PDFJS.md)
- WASM implementation: [README.md](README.md)
- PDF.js library: https://mozilla.github.io/pdf.js/
