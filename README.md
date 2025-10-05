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

The build will be stored in the `pkg` directory in the project root.
