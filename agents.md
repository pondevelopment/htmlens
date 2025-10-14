# AI Agent Guide for htmlens

This document provides context and guidelines for AI agents working with the htmlens codebase. It's designed to help AI assistants understand the project structure, make effective contributions, and maintain code quality.

## Project Overview

**htmlens** is a Rust-based command-line tool that extracts semantic knowledge graphs and readable text from HTML pages. It reveals structured data (JSON-LD/Schema.org) embedded in web pages, making it valuable for SEO analysis, web scraping, and understanding how search engines interpret content.

### Core Purpose
- Extract JSON-LD structured data from HTML
- Build knowledge graphs from Schema.org entities
- Convert HTML to clean Markdown
- Visualize entity relationships with Mermaid diagrams
- Identify DataDownload resources

## Architecture

### Single-File Structure
The entire application is in `src/main.rs` (~1,771 lines). While monolithic, it's organized into logical sections:

1. **CLI Parsing** (lines 1-200): Argument handling and command routing
2. **Main Execution** (lines 201-350): Orchestrates fetching, parsing, and output
3. **Data Structures** (lines 400-500): Core types for graphs and insights
4. **Graph Building** (lines 850-1100): Processes JSON-LD into knowledge graphs
5. **Insights Extraction** (lines 500-850): Analyzes graphs for product/variant data
6. **Output Rendering** (lines 1200-1600): Formats results as tables, summaries, diagrams
7. **Utility Functions** (lines 1600-1771): Helpers for formatting, escaping, path handling

### Key Data Flow
```
URL → fetch() → HTML
  ↓
extract_json_ld_blocks() → JSON-LD strings
  ↓
expand_block() → Expanded JSON-LD documents
  ↓
GraphBuilder → KnowledgeGraph (nodes + edges)
  ↓
GraphInsights::from() → Analyzed data
  ↓
render_*() functions → Formatted output
```

## Core Data Structures

### KnowledgeGraph
```rust
struct KnowledgeGraph {
    nodes: Vec<GraphNode>,  // All entities in the graph
    edges: Vec<GraphEdge>,  // Relationships between entities
}
```

### GraphNode
```rust
struct GraphNode {
    id: String,                           // IRI or blank node ID
    types: Vec<String>,                   // @type values (e.g., "Product")
    properties: HashMap<String, JsonValue>, // Literal properties
}
```

### GraphEdge
```rust
struct GraphEdge {
    from: String,      // Source node ID
    to: String,        // Target node ID
    predicate: String, // Relationship type (e.g., "offers", "brand")
}
```

### GraphInsights
```rust
struct GraphInsights {
    product_group: Option<ProductGroupSummary>,
    variants: Vec<VariantSummary>,
    graph_summary: Vec<String>,
    data_downloads: Vec<DataDownloadEntry>,
}
```

## Key Functions Reference

### Entry Points
- `main()`: Parses CLI args, routes to help/version/run
- `run(options)`: Main execution flow
- `parse_arguments(&[String])`: CLI argument parser

### Fetching & Parsing
- `fetch(url) → Result<String>`: HTTP GET request
- `extract_json_ld_blocks(html) → Result<Vec<String>>`: Finds `<script type="application/ld+json">`
- `expand_block(base_url, json_ld, loader) → Result<ExpandedDocument>`: JSON-LD expansion

### Graph Construction
- `GraphBuilder::new()`: Creates empty graph builder
- `GraphBuilder::ingest_document(doc)`: Processes expanded JSON-LD
- `GraphBuilder::process_node(node) → String`: Recursively processes nodes
- `GraphBuilder::into_graph() → KnowledgeGraph`: Finalizes the graph

### Analysis
- `GraphInsights::from(graph) → GraphInsights`: Extracts product/variant insights
- `summarize_variant(product, adjacency, nodes, ...) → VariantSummary`: Analyzes product variants
- `extract_offer(product, adjacency, nodes) → Option<OfferInfo>`: Finds pricing/availability
- `collect_additional_properties(product, adjacency, nodes) → HashMap<String, String>`: Extracts PropertyValue entities

### Rendering
- `render_variant_table(buf, variants, total)`: Creates markdown table
- `render_graph_summary(buf, lines)`: Outputs condensed relationships
- `render_data_downloads_section(buf, entries)`: Lists DataDownload resources
- `graph_to_mermaid(graph) → String`: Generates Mermaid diagram

### Utilities
- `property_text(node, keys) → Option<String>`: Extracts text from node properties (handles schema.org variations)
- `has_schema_type(node, type_name) → bool`: Checks if node has specific @type
- `shorten_iri(iri) → String`: Converts full IRI to short name
- `format_price(value, currency) → String`: Formats prices with symbols
- `sanitize_html_for_markdown(html) → String`: Removes scripts/styles before conversion
- `build_output_path(base, url) → PathBuf`: Generates filename from URL

## Common Patterns

### Property Resolution
Properties are checked with multiple key variations (http/https, full IRI, short name):
```rust
property_text(node, &[
    "https://schema.org/name",
    "http://schema.org/name",
    "name"
])
```

### Schema.org Type Checking
Types are compared case-insensitively after shortening:
```rust
has_schema_type(node, "Product")  // Matches "https://schema.org/Product"
```

### Edge Traversal
Build adjacency map for efficient traversal:
```rust
let adjacency = build_adjacency(graph);
if let Some(edges) = adjacency.get(node_id) {
    for edge in edges {
        if predicate_matches(&edge.predicate, "offers") {
            // Process offer relationship
        }
    }
}
```

## CLI Options

### Flags
- `-g, --graph-only`: Output condensed graph summary only
- `-G, --graph-summary`: Include product summaries and graph
- `-dd, --data-downloads`: Include DataDownload references
- `-s, --save [PATH]`: Save output to file
- `-v, --version`: Show version
- `-h, --help`: Show help

### Output Modes
1. **Default**: Markdown only
2. **--graph-summary**: Markdown + product tables + graph summary + data downloads
3. **--graph-only**: Just condensed graph relationships

## Development Guidelines

### When Adding Features

1. **New CLI Options**
   - Update `OutputMode` enum if adding new modes
   - Modify `parse_arguments()` to handle new flags
   - Update `print_help()` with documentation
   - Add logic in `run()` to use the new option

2. **New Schema.org Types**
   - Add detection in `GraphInsights::from()`
   - Create summary struct if needed (like `ProductGroupSummary`)
   - Add rendering function for new type
   - Update graph summary generation

3. **New Output Formats**
   - Add conversion function (like `graph_to_mermaid`)
   - Add render function (like `render_variant_table`)
   - Update `run()` to call new renderer based on options

4. **New Property Extractors**
   - Follow the `property_text()` pattern for consistency
   - Always check multiple key variations (http/https, full/short)
   - Handle both single values and arrays

### Code Style

- **Error Handling**: Use `anyhow::Result` and `.context()` for descriptive errors
- **Async**: Use `tokio::main` for async entry point
- **Formatting**: Use `cargo fmt` before committing
- **Naming**: Snake_case for functions/variables, PascalCase for types
- **Documentation**: Add doc comments for complex functions

### Testing Approach

Currently no unit tests. When adding tests:
- Test property extraction with various key formats
- Test JSON-LD expansion with sample documents
- Test edge cases in URL parsing and filename generation
- Mock HTTP requests for fetch testing

## Common Modifications

### Adding a New Product Property

1. Add field to `VariantSummary`:
```rust
struct VariantSummary {
    // ... existing fields
    new_property: Option<String>,
}
```

2. Extract in `summarize_variant()`:
```rust
let mut summary = VariantSummary {
    // ... existing fields
    new_property: property_text(
        product,
        &["https://schema.org/newProperty", "newProperty"]
    ),
};
```

3. Add column to table in `render_variant_table()`:
```rust
let headers = ["SKU", "Color", "Size", "NewProperty", "Price", "Availability"];
// ... add to row rendering
```

### Adding Support for a New Entity Type

1. Create summary struct:
```rust
struct NewEntitySummary {
    name: Option<String>,
    // ... other fields
}
```

2. Add to `GraphInsights`:
```rust
struct GraphInsights {
    // ... existing fields
    new_entities: Vec<NewEntitySummary>,
}
```

3. Detect and process in `GraphInsights::from()`:
```rust
if let Some(entity) = nodes_map
    .values()
    .find(|node| has_schema_type(node, "NewEntity"))
{
    // Extract data and add to insights.new_entities
}
```

4. Add rendering function and call from `run()`.

### Modifying Output Format

All output is built in the `run()` function using `String` buffer and `writeln!` macro. Sections are controlled by boolean flags:
- `include_markdown`
- `include_summary_sections`
- `include_condensed_summary`
- `include_graph_exports`

Add new sections by:
1. Adding conditional block in `run()`
2. Using helper functions like `push_section_header()`
3. Building content with `writeln!()` or dedicated render functions

## Dependencies

### Core Libraries
- **reqwest**: HTTP client with async support
- **scraper**: HTML parsing with CSS selectors
- **html2md**: HTML to Markdown conversion
- **json-ld**: JSON-LD expansion and processing
- **serde/serde_json**: Serialization
- **tokio**: Async runtime
- **anyhow**: Error handling
- **url**: URL parsing
- **once_cell**: For lazy static initialization
- **regex**: For efficient text processing

### Version Constraints
- Rust edition: 2024
- Minimum Rust version: 1.85
- See `Cargo.toml` for specific dependency versions

## Troubleshooting

### Common Issues

1. **JSON-LD Expansion Fails**
   - Check if base URL is valid IRI
   - Verify remote contexts are accessible
   - Look for malformed JSON in script tags

2. **Property Not Extracted**
   - Verify key variations (http vs https)
   - Check if property is literal vs. node reference
   - Use `resolve_node_property()` pattern

3. **Graph Relationships Missing**
   - Ensure node IDs match between edges and nodes
   - Check predicate matching logic
   - Verify JSON-LD expansion included relationships

4. **Output Formatting Issues**
   - Check string escaping (Mermaid, HTML, Markdown)
   - Verify table column alignment
   - Test with edge cases (empty values, special characters)

## Future Improvements to Consider

### Refactoring Opportunities
- Split `main.rs` into modules (cli, graph, insights, render, utils)
- Add unit tests for core functions
- Create trait abstractions for extensibility
- Add integration tests with sample HTML

### Feature Ideas
- Batch processing multiple URLs
- JSON/CSV output formats
- Interactive filtering/querying
- Support for more Schema.org types (Article, Event, Recipe, etc.)
- Extract OpenGraph/Twitter Card metadata
- Diff mode to compare two pages
- Watch mode for monitoring changes
- Plugin system for custom extractors

### Performance Optimizations
- Parallel processing for multiple JSON-LD blocks
- Caching of remote contexts
- Streaming output for large graphs
- Memory-efficient graph representation

## Getting Help

When working on this codebase:
1. Read the README.md for user-facing documentation
2. Check function signatures and inline comments
3. Use `cargo doc --open` for dependency documentation
4. Test changes with real-world URLs (e-commerce sites work well)
5. Run `cargo clippy` for linting suggestions

## Contributing Guidelines

When making changes:
1. Create a feature branch: `git checkout -b feature/description`
2. Make focused, logical commits
3. Update README.md if adding user-facing features
4. Test with multiple real-world URLs
5. Run `cargo fmt` and `cargo clippy`
6. Create a pull request with clear description

---

**Remember**: This tool is designed for marketers and developers to understand what search engines and AI agents see. Keep the output human-readable and the code maintainable.
