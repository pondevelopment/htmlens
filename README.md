# HTML Lens

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
- **json-ld** - JSON-LD expansion and context resolution
- **serde & serde_json** - Serialization/deserialization
- **tokio** - Async runtime for concurrent operations

## Project Structure

```
htmlens/
├── Cargo.toml           # Project metadata and dependencies
├── src/
│   ├── main.rs          # CLI interface and output formatting
│   ├── parser.rs        # HTML fetching, parsing, and JSON-LD extraction
│   └── ld_graph.rs      # JSON-LD expansion and knowledge graph building
├── LICENSE              # MIT License
└── README.md            # This file
```

## Prerequisites

- Rust 1.70 or newer (stable toolchain recommended).
- Network access from the environment where the binary runs.

## Build

```bash
cargo build --release
```

## How to Run

### Basic Usage

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
- **Check compilation**: `cargo check`
- **Build optimized binary**: `cargo build --release`

### Adding New Features

When adding new features:
1. **Parser module** (`src/parser.rs`): Add new HTML/JSON-LD parsing or fetching capabilities
2. **LD Graph module** (`src/ld_graph.rs`): Extend JSON-LD expansion or graph building logic
3. **Main module** (`src/main.rs`): Update CLI arguments, output formatting, or entity extraction
4. Update documentation in this README
5. Test with various real-world URLs and JSON-LD inputs

## Implementation Notes

- **Modular architecture**: Code is organized into three main modules:
  - `parser`: HTTP fetching, HTML sanitization, and JSON-LD extraction
  - `ld_graph`: JSON-LD expansion and knowledge graph construction
  - `main`: CLI interface, entity extraction, and output formatting
- Uses `reqwest` for HTTP with custom user agent, `html2md` to generate Markdown, and `scraper` to
  locate `application/ld+json` blocks.
- JSON‑LD expansion relies on the `json-ld` crate's `ReqwestLoader` to resolve
  remote contexts.
- The graph builder normalizes node identifiers, collects literal properties,
  and tracks edges (`offers`, `brand`, `hasVariant`, `isVariantOf`, etc.) between nodes.
- `DataDownload` entities are detected by scanning the expanded document and
  collecting `contentUrl` values regardless of compacted or expanded keys.
- **Multiple JSON-LD blocks** in a single HTML page are automatically combined into a single `@graph` structure.
- **Property inheritance**: Variants referencing other products via `isVariantOf` inherit properties not explicitly overridden.
- **Common properties** are dynamically extracted from the first variant and filtered against the `variesBy` list using intelligent substring matching.
- Supports both **FrameShape** and **FrameType** property names for bicycle frame specifications.

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
