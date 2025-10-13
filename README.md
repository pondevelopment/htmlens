# HTML Lens

See the semantic web through a clear lens.

`htmlens` reveals the structured reality hiding inside raw HTML by expanding
JSON‑LD, mapping Schema.org entities, and presenting the inferred knowledge
graph alongside the source content.

- A Markdown representation of the page.
- A JSON serialization of extracted graph nodes and relationships.
- A Mermaid diagram describing entity connections.
- A list of any `DataDownload` `contentUrl` values discovered in the graph.

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
│   └── main.rs          # Main application logic
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
htmlens [OPTIONS] <url>
```

### Examples

Extract and display page content as Markdown:

```bash
htmlens https://example.com/product-page
```

Include graph summary and save to file:

```bash
htmlens https://example.com/product-page --graph-summary --save reports
```

Show only the knowledge graph:

```bash
htmlens https://example.com/product-page --graph-only
```

Extract data downloads:

```bash
htmlens https://example.com/dataset --data-downloads
```

### Running with Cargo

```bash
cargo run --release --bin htmlens -- <url> [OPTIONS]
```

The program emits the Markdown, JSON, Mermaid graph, and data download URLs to
`stdout` so they can be saved or piped as needed.

### CLI Flags

- `-g`, `--graph-only` &mdash; Output only the condensed graph summary.
- `-G`, `--graph-summary` &mdash; Include product summaries and condensed graph.
- `-dd`, `--data-downloads` &mdash; Show detected `DataDownload` entries with
  their URLs, encoding formats, and licenses.
- `-s`, `--save [path]` &mdash; Write the output to disk. Provide a directory or
  explicit filename (`.md`) to control where the report is stored. Without a
  value, the tool writes to the current working directory using a name derived
  from the URL.
- `-v`, `--version` &mdash; Show version information.
- `-h`, `--help` &mdash; Show this help message.

### Output Modes

- *Default* (no flags) — render only the page Markdown.
- `--graph-summary` — render Markdown plus product summaries, variant table,
  data downloads, and condensed graph relationships.
- `--graph-only` — show just the condensed graph summary (and optional
  DataDownload section when `-dd` is present).

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
1. Update the CLI argument parsing in `main.rs`
2. Add corresponding logic for new extraction or formatting capabilities
3. Update documentation in this README
4. Test with various real-world URLs

## Implementation Notes

- Uses `reqwest` for HTTP, `html2md` to generate Markdown, and `scraper` to
  locate `application/ld+json` blocks.
- JSON‑LD expansion relies on the `json-ld` crate’s `ReqwestLoader` to resolve
  remote contexts.
- The graph builder normalizes node identifiers, collects literal properties,
  and tracks edges (`offers`, `brand`, `hasVariant`, etc.) between nodes.
- `DataDownload` entities are detected by scanning the expanded document and
  collecting `contentUrl` values regardless of compacted or expanded keys.

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

Built for Marketeers and Developers to understand better what an Agent or Screper sees.
