# page-counter-wasm

A page counter module for uploaded documents with two implementations:
- **WASM (Rust)**: Multi-format support (PDF, XLSX, TXT, MD) with excellent performance
- **PDF.js (JavaScript)**: PDF-only with simple npm setup

## Implementations

### 1. WASM Implementation (Rust)

The main implementation using Rust compiled to WebAssembly for optimal performance and multi-format support.

#### pre-requisites

Install the `wasm-pack` crate.

```bash
cargo install wasm-pack
```

#### build

```bash
wasm-pack build --target web
```

The build will be stored in the `pkg` directory in the project root. You can then import the ts/js files and the wasm build into your frontend. `index.html` contains a demo frontend for this purpose.

#### demo

To run the WASM demo frontend, serve it via an http-server

```bash
# using python
# in the project root
python3 -m http.server
# will start a server on port 8000
# then open http://localhost:8000/index.html
```

### 2. PDF.js Implementation (JavaScript)

A JavaScript-based PDF counter using Mozilla's PDF.js library. Simpler setup but PDF-only.

#### installation

```bash
npm install
```

#### usage

See `counter.js` for the full API. Quick example:

```javascript
import { countPdfPages, getPdfInfo } from './counter.js';

const file = document.querySelector('input[type="file"]').files[0];
const arrayBuffer = await file.arrayBuffer();

// Count pages
const pageCount = await countPdfPages(arrayBuffer);
console.log(`PDF has ${pageCount} pages`);

// Get detailed info
const info = await getPdfInfo(arrayBuffer);
console.log('Page details:', info);
```

#### demo

A demo is available at `demo-pdfjs.html`:

```bash
npm run dev
# or
python3 -m http.server 8000
# then open http://localhost:8000/demo-pdfjs.html
```

#### Node.js example

```bash
node example-node.mjs path/to/your/file.pdf
```

For complete documentation, see [README-PDFJS.md](README-PDFJS.md)

## Which Implementation Should I Use?

| Feature | PDF.js (JavaScript) | WASM (Rust) |
|---------|-------------------|-------------|
| **Setup** | `npm install` | `cargo install wasm-pack` + build |
| **Bundle Size** | ~1.5MB | ~200KB |
| **Format Support** | PDF only | PDF, XLSX, TXT, MD |
| **Performance** | Good | Excellent |
| **Dependencies** | Node.js/npm | Rust toolchain |

**Use PDF.js if:**
- You only need PDF support
- You prefer simpler npm-based setup
- You're already using JavaScript/Node.js

**Use WASM if:**
- You need multi-format support
- You want smaller bundle size
- You need maximum performance
