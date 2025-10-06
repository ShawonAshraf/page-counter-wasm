# page-counter-wasm

A rust web-assembly module to count pages from uploaded documents (pdf, xlsx, txt and markdown) in the frontend. 

## pre-requisites

Install the `wasm-pack` crate.

```bash
cargo install wasm-pack
```

## build

```bash
wasm-pack build --target web
```

The build will be stored in the `pkg` directory in the project root. You can then import the ts/js files and the wasm build into your frontend. `index.html` contains a demo frontend for this purpose.

To run the demo frontend, serve it via an http-server

```bash
# using python
# in the project root
python3 -m http.server
# will start a server on port 8000
```
