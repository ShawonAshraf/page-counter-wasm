# page-counter-wasm

A rust web-assembly module to count pages from uploaded documents (pdf, xlsx, txt and markdown) in the frontend. The
module embeds pdfjs inside wasm to parse pdf and pure rust methods for excel, txt and markdown file parsing.

## pre-requisites

Install the `wasm-pack` crate.

```bash
cargo install wasm-pack
```

## build

```bash
wasm-pack build --target web
```

The build will be stored in the `pkg` directory in the project root. You can then import the ts/js files and the wasm
build into your frontend. `views/wasm.html` contains a demo frontend for this purpose.

## running the frontend

To run the demo frontend

```bash
npm run serve
# will start a server on port 8000
```

Then visit `http://localhost:8000/views/wasm.html`.

## pdfjs implmentation (for comparison)

Once you've started the server, visit `http://localhost:8000/views/pdfjs.html`. 
