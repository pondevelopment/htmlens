# htmlens-core

Core library for extracting semantic knowledge graphs from HTML pages.

## Features

- **HTML Parsing**: Extract JSON-LD blocks from HTML documents
- **JSON-LD Processing**: Combine multiple JSON-LD blocks with context hoisting
- **HTML Sanitization**: Remove scripts, styles, and unwanted elements
- **Markdown Conversion**: Convert clean HTML to Markdown
- **JSON-LD Expansion** (optional): Full JSON-LD expansion with remote context resolution

## Feature Flags

- `default`: Basic HTML parsing and JSON-LD extraction (lightweight, no async dependencies)
- `full-expansion`: Complete JSON-LD expansion with remote context resolution (requires `reqwest`, `tokio`, `json-ld`)

## Usage

### Basic Usage (no expansion)

```rust
use htmlens_core::parser;

let html = r#"
    <script type="application/ld+json">
    {"@context": "https://schema.org", "@type": "Product", "name": "Example"}
    </script>
"#;

let blocks = parser::extract_json_ld_blocks(html)?;
let json_ld = parser::combine_json_ld_blocks(blocks)?;
println!("{}", json_ld);
```

### With Full Expansion

```toml
[dependencies]
htmlens-core = { version = "0.4", features = ["full-expansion"] }
```

```rust
use htmlens_core::{parser, graph::{expand_json_ld, GraphBuilder}};
use json_ld::ReqwestLoader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let html = parser::fetch_html("https://example.com").await?;
    let blocks = parser::extract_json_ld_blocks(&html)?;
    let combined = parser::combine_json_ld_blocks(&blocks)?;
    
    let mut loader = ReqwestLoader::default();
    let expanded = expand_json_ld("https://example.com", &combined, &mut loader).await?;
    
    let mut builder = GraphBuilder::new();
    builder.ingest_document(&expanded);
    let graph = builder.into_graph();
    
    println!("Nodes: {}", graph.nodes.len());
    println!("Edges: {}", graph.edges.len());
    Ok(())
}
```

## Architecture

- `parser.rs`: HTML fetching, JSON-LD extraction, sanitization, and Markdown conversion
- `graph.rs`: JSON-LD expansion and knowledge graph construction (requires `full-expansion` feature)
- `lib.rs`: Public API and re-exports

## License

MIT
