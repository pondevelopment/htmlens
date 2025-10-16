# HTMLens

See the semantic web through a clear lens.

`htmlens` reveals the structured reality hiding inside raw HTML by expanding
JSON‑LD, mapping Schema.org entities, and presenting the inferred knowledge
graph alongside the source content. It can also accept JSON-LD directly for
offline analysis.

**Features:**
- Extract and visualize Schema.org structured data from web pages or direct JSON-LD input
- Markdown representation of page content
- Comprehensive product information with variants, pricing, and availability
- Dynamic extraction of common (non-varying) properties across product variants
- Support for multiple ProductGroups with property inheritance
- JSON serialization of graph nodes and relationships
- Mermaid diagram generation for entity connections
- Detection of `DataDownload` resources
- **NEW**: Cloudflare Worker API with beautiful web interface

## Topics Covered

- **Semantic Web & Linked Data**: Extract and visualize JSON-LD structured data embedded in HTML pages
- **Schema.org Mapping**: Parse and interpret Schema.org entities and their relationships
- **Knowledge Graph Extraction**: Build graph representations from web content
- **Web Scraping & SEO**: Understand how search engines and crawlers interpret your pages
- **Data Extraction**: Identify and collect `DataDownload` resources with metadata

## Technologies

- **Rust** - High-performance, memory-safe systems programming language
- **reqwest** - HTTP client for fetching web pages
- **scraper** - HTML parsing and CSS selector support
- **html2md** - Convert HTML to clean Markdown
- **json-ld** - JSON-LD expansion and context resolution (full-expansion feature)
- **serde & serde_json** - Serialization/deserialization
- **tokio** - Async runtime for concurrent operations
- **Cloudflare Workers** - Edge computing platform for the web API

## Project Structure

```
htmlens/                         # Cargo workspace v0.4.0
├── Cargo.toml                   # Workspace definition
├── crates/
│   ├── htmlens-core/           # 🔧 Core library (reusable)
│   │   ├── Cargo.toml          # Feature flags: default, full-expansion
│   │   ├── src/
│   │   │   ├── lib.rs          # Public API with conditional exports
│   │   │   ├── types.rs        # Core types (always available)
│   │   │   ├── parser.rs       # HTML/JSON-LD parsing (always available)
│   │   │   └── graph.rs        # Graph building (full-expansion only)
│   │   └── README.md
│   ├── htmlens-cli/            # 📦 Command-line tool
│   │   ├── Cargo.toml          # Uses full-expansion feature
│   │   ├── src/
│   │   │   └── main.rs         # CLI interface (~2200 lines)
│   │   └── README.md
│   └── htmlens-worker/         # ☁️ Cloudflare Worker
│       ├── Cargo.toml          # Lightweight (no full-expansion)
│       ├── src/
│       │   ├── lib.rs          # Worker API (~440 lines)
│       │   └── frontend.html   # Web UI (~215 lines)
│       ├── wrangler.toml       # CF Worker config
│       ├── package.json        # Node.js dependencies
│       ├── .nvmrc              # Node v22
│       └── README.md
├── reports/                     # Example outputs
├── LICENSE                      # MIT License
├── README.md                    # This file
└── AGENTS.md                    # AI agent development guide
```

### Feature Flags

The `htmlens-core` library uses feature flags to manage dependencies:

- **`default`**: Lightweight mode with basic HTML/JSON-LD extraction
  - Includes: parser, types, HTML sanitization, markdown conversion
  - No JSON-LD expansion or heavy dependencies
  
- **`full-expansion`**: Complete functionality with JSON-LD expansion
  - Includes: All default features + JSON-LD expansion + graph building
  - Dependencies: json-ld, reqwest, tokio, uuid
  - Used by: `htmlens-cli`
  - Not used by: `htmlens-worker` (keeps WASM bundle small)

## Prerequisites

- Rust 1.85 or newer (2024 edition)
- Network access for fetching remote web pages

## Build

**Build the entire workspace:**
```bash
cargo build --release --workspace
```

**Build specific components:**
```bash
# CLI only
cargo build --release -p htmlens-cli

# Core library
cargo build --release -p htmlens-core

# Cloudflare Worker
cargo build --release -p htmlens-worker
```

**Install CLI globally:**
```bash
cargo install --path crates/htmlens-cli
```

## How to Run

### Command-Line Interface

```bash
htmlens --help
```

```bash
htmlens [OPTIONS] <URL|JSON-LD>
```

**Input Options:**
- **URL**: Fetch and extract JSON-LD from a web page
- **JSON-LD**: Provide JSON-LD directly as a string (must start with `{` or `[`)

### Examples

Extract and display page content with summaries (default):

```bash
htmlens https://example.com/product-page
```

Process JSON-LD directly:

```bash
htmlens '{"@context": "https://schema.org", "@type": "Product", "name": "Example"}'
```

Include Mermaid diagram visualization:

```bash
htmlens https://example.com/product-page --mermaid
```

Show only the knowledge graph summary:

```bash
htmlens https://example.com/product-page --graph-only
```

Extract with data downloads and save to file:

```bash
htmlens https://example.com/dataset --data-downloads --save reports
```

### Running with Cargo

```bash
cargo run --release -- <URL|JSON-LD> [OPTIONS]
```

The program outputs to `stdout` in the following order:
1. **Markdown** representation of the source page (for URL input)
2. **Structured summaries** including:
   - Organization details (name, contact, address, ratings)
   - Contact points (phone, email)
   - Breadcrumb navigation
   - Product/ProductGroup information with:
     - Common properties (shared by all variants)
     - Variant tables (SKU, color, size, price, availability, etc.)
     - Dynamic property extraction based on `variesBy` field
3. **Data downloads** (when `-dd` flag is used)
4. **Knowledge graph** visualization (when `-m` flag is used):
   - JSON representation of all graph nodes and edges
   - Mermaid diagram for visual exploration

### CLI Flags

- `-g`, `--graph-only` &mdash; Output only the condensed graph summary (no markdown or product details).
- `-G`, `--graph-summary` &mdash; Output product summaries only (no markdown).
- `-m`, `--mermaid` &mdash; Include Mermaid diagram visualization of the knowledge graph with JSON export.
- `-dd`, `--data-downloads` &mdash; Show detected `DataDownload` entries with
  their URLs, encoding formats, and licenses.
- `-s`, `--save [path]` &mdash; Write the output to disk. Provide a directory or
  explicit filename (`.md`) to control where the report is stored. Without a
  value, the tool writes to the current working directory using a name derived
  from the URL.
- `-v`, `--version` &mdash; Show version information.
- `-h`, `--help` &mdash; Show this help message.

### Output Modes

- **Default** (no flags) — Markdown + product summaries with common properties and variant details.
- `--graph-summary` or `-G` — Same as default (alias for backwards compatibility).
- `--graph-only` or `-g` — Show just the condensed graph summary (no markdown, no product details).

### Key Features

#### Multiple ProductGroup Support
Processes **all** ProductGroups found in the JSON-LD data, not just the first one. Each ProductGroup is displayed with its own variants and statistics.

#### Dynamic Common Properties
Automatically extracts and displays properties that are shared by all variants but not in the `variesBy` list. This includes:
- Product descriptions
- Materials
- Motor specifications
- Battery inclusion status
- Any custom `additionalProperty` items

#### Property Inheritance
Variants using `isVariantOf` to reference parent products automatically inherit properties like frame type, motor brand, etc., ensuring complete information even when properties aren't duplicated.

#### Smart Property Filtering
The tool intelligently filters properties based on `variesBy` using substring matching. For example, if `variesBy` includes "FrameSize", the tool won't show "size" as a common property.

## Development

### Setting Up Development Environment

1. **Clone the repository**:
   ```bash
   git clone https://github.com/pondevelopment/htmlens.git
   cd htmlens
   ```

2. **Build the project**:
   ```bash
   cargo build
   ```

3. **Run tests** (if available):
   ```bash
   cargo test
   ```

4. **Run in development mode**:
   ```bash
   cargo run -- <url> [OPTIONS]
   ```

### Development Workflow

- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`
- **Check compilation**: `cargo check --workspace`
- **Build optimized binary**: `cargo build --release --workspace`

### Adding New Features

When adding new features:
1. **Core library** (`crates/htmlens-core`): Add new HTML/JSON-LD parsing or graph building capabilities
2. **CLI tool** (`crates/htmlens-cli`): Update CLI arguments, output formatting, or entity extraction
3. **Worker** (`crates/htmlens-worker`): Add new API endpoints or modify extraction logic
4. Update documentation in respective README files
5. Test with various real-world URLs and JSON-LD inputs

### Cloudflare Worker Deployment

The `htmlens-worker` crate provides a **lightweight web API** with a beautiful interface for JSON-LD extraction:

**Features:**
- 🎨 Beautiful gradient UI (purple/blue theme)
- 📊 Business Summary with product information and technical insights
- 🔍 JSON-LD tab with syntax highlighting and combined `@graph` structure
- 📋 Structured Data tab with CLI-style product tables
- 📄 Page Content tab with clean markdown conversion
- 🚀 Fast edge computing with Cloudflare Workers
- 🌐 CORS-enabled API for integration

**Local Development:**
```bash
cd crates/htmlens-worker

# Install dependencies (Node.js v22 required, see .nvmrc)
npm install

# Run locally
npx wrangler dev
```

**Deploy to Cloudflare:**
```bash
npm run deploy
```

**Web Interface:**
Visit `http://localhost:8787` (local) or your deployed worker URL to access the interactive web interface.

**API Usage:**
```bash
# Analyze a URL
curl "https://your-worker.workers.dev/?url=https://example.com/product"

# Health check
curl "https://your-worker.workers.dev/health"
```

**API Response:**
```json
{
  "url": "https://example.com/product",
  "title": "Product Page Title",
  "description": "Page description",
  "graph": {
    "nodes": [...],
    "edges": [...]
  },
  "jsonld": [...],           // Raw blocks array
  "jsonldGraph": {           // Combined @graph structure
    "@context": "https://schema.org",
    "@graph": [...]
  },
  "markdown": "...",         // CLI-style formatted tables
  "pageMarkdown": "...",     // HTML converted to markdown
  "meta": {
    "htmlLength": 173130,
    "jsonldCount": 4,
    "wasmStatus": "rust"
  }
}
```

See `crates/htmlens-worker/README.md` for detailed API documentation.

## Implementation Notes

- **Workspace architecture**: Organized as a Cargo workspace with three crates:
  - `htmlens-core`: Reusable library with feature flags for lightweight vs. full functionality
  - `htmlens-cli`: Command-line interface with full features (Markdown, tables, Mermaid, JSON-LD expansion)
  - `htmlens-worker`: Cloudflare Worker with web UI and API (lightweight, no JSON-LD expansion)
- **Feature-gated dependencies**: Heavy dependencies like `json-ld`, `reqwest`, `tokio`, and `uuid` are only included when the `full-expansion` feature is enabled
- **WASM compatibility**: Worker uses `getrandom` with `wasm_js` feature for random number generation in WebAssembly
- Uses `reqwest` for HTTP with custom Mozilla user agent for better compatibility
- `html2md` generates clean Markdown, and `scraper` locates `application/ld+json` blocks
- JSON‑LD expansion (in CLI only) relies on the `json-ld` crate's `ReqwestLoader` to resolve remote contexts
- **Multiple JSON-LD blocks** in a single HTML page are automatically combined into a single `@graph` structure with shared `@context`
- The graph builder normalizes node identifiers, collects literal properties, and tracks edges (`offers`, `brand`, `hasVariant`, `isVariantOf`, etc.) between nodes
- `DataDownload` entities are detected by scanning the expanded document and collecting `contentUrl` values
- **Property inheritance**: Variants referencing other products via `isVariantOf` inherit properties not explicitly overridden
- **Common properties** are dynamically extracted from the first variant and filtered against the `variesBy` list using intelligent token-based matching
- **Token-based property matching** prevents false positives (e.g., "color" won't match "colorway")
- Supports both **FrameShape** and **FrameType** property names for bicycle frame specifications
- **Worker frontend**: Beautiful gradient UI with syntax-highlighted JSON-LD, HTML tables for structured data, and business summaries

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! If you have suggestions, bug reports, or want to add features, please open an issue or submit a pull request.

To contribute:
- Fork the repository and create your branch from `main`.
- Make your changes with clear commit messages.
- Ensure the code builds and passes any tests.
- Open a pull request describing your changes.

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Built for Marketeers and Developers to understand better what an Agent or Scraper sees.

**Developed by Pon Datalab**
