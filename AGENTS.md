# AI Agent Guide for htmlens

This document provides context and guidelines for AI agents working with the htmlens codebase. It's designed to help AI assistants understand the project structure, make effective contributions, and maintain code quality.

## Project Overview

**htmlens** is a Rust-based command-line tool that extracts semantic knowledge graphs and readable text from HTML pages. It reveals structured data (JSON-LD/Schema.org) embedded in web pages, making it valuable for SEO analysis, web scraping, and understanding how search engines interpret content.

### Core Purpose
- Extract JSON-LD structured data from HTML or accept direct JSON-LD input
- Build knowledge graphs from Schema.org entities
- Convert HTML to clean Markdown
- Analyze product variants and extract common properties
- Visualize entity relationships with Mermaid diagrams
- Identify DataDownload resources

### Version & Status
- **Current Version**: 0.3.0
- **Rust Edition**: 2024
- **License**: MIT
- **Repository**: https://github.com/pondevelopment/htmlens

## Architecture

### Modular Structure

The project is organized into **three main modules** for maintainability:

```
src/
├── main.rs      (~1,943 lines) - CLI interface and output formatting
├── parser.rs    (~164 lines)   - HTML fetching and JSON-LD extraction
└── ld_graph.rs  (~351 lines)   - JSON-LD expansion and graph building
```

### Module Responsibilities

#### `src/parser.rs` - Input Processing
**Purpose**: Handle HTML fetching, sanitization, and JSON-LD extraction

**Key Functions**:
- `fetch_html(url: &str) -> Result<String>` - HTTP client with custom user agent
- `extract_json_ld_blocks(html: &str) -> Result<Vec<String>>` - Finds `<script type="application/ld+json">` tags
- `combine_json_ld_blocks(blocks: Vec<String>) -> Result<String>` - Merges multiple JSON-LD blocks, handles arrays
- `sanitize_html(html: &str) -> String` - Removes scripts, styles, and comments
- `html_to_markdown(html: &str) -> String` - Converts sanitized HTML to Markdown

**Design Notes**:
- Uses `reqwest` with Mozilla user agent for better compatibility
- Flexible JSON-LD detection with `.contains("ld+json")`
- Hoists first `@context` when merging multiple blocks
- Handles both object and array-based JSON-LD documents

#### `src/ld_graph.rs` - Graph Construction
**Purpose**: Expand JSON-LD and build knowledge graph structures

**Key Structures**:
```rust
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphNode {
    pub id: String,                              // IRI or blank node ID
    pub types: Vec<String>,                      // @type values
    pub properties: HashMap<String, JsonValue>,  // Literal properties
}

pub struct GraphEdge {
    pub from: String,      // Source node ID
    pub to: String,        // Target node ID  
    pub predicate: String, // Relationship type
}

pub struct GraphBuilder {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
}
```

**Key Functions**:
- `expand_json_ld(base_url: &str, json_ld: &str, loader: &ReqwestLoader) -> Result<ExpandedDocument>` - Expands JSON-LD with context resolution
- `GraphBuilder::new() -> Self` - Creates empty builder
- `GraphBuilder::ingest_document(&mut self, doc: &ExpandedDocument)` - Processes expanded JSON-LD
- `GraphBuilder::process_node(&mut self, node: &Object) -> String` - Recursively processes nodes
- `GraphBuilder::into_graph(self) -> KnowledgeGraph` - Finalizes the graph

**Design Notes**:
- Uses `json-ld` crate (v0.17.2) for standards-compliant expansion
- Handles nested structures and blank nodes
- Separates literals (properties) from object references (edges)
- Resolves remote contexts asynchronously

#### `src/main.rs` - CLI & Output
**Purpose**: Command-line interface, entity extraction, and formatted output

**Key Sections**:
1. **CLI Parsing** (~lines 45-195): Argument handling and command routing
2. **Main Execution** (~lines 195-350): Orchestrates the data flow
3. **Data Structures** (~lines 350-600): Insights, summaries, and entity types
4. **Analysis** (~lines 600-1000): Extract product/variant information
5. **Rendering** (~lines 1000-1700): Format output as tables, summaries, diagrams
6. **Utilities** (~lines 1700-1943): Helpers for formatting, escaping, path handling

**Key Types**:
```rust
enum OutputMode {
    Default,      // Markdown + summaries
    SummaryOnly,  // Product summaries only (no markdown)
    GraphOnly,    // Only condensed graph summary
}

enum InputSource {
    Url(String),
    JsonLd(String),
}

struct GraphInsights {
    product_group: Option<ProductGroupSummary>,
    variants: Vec<VariantSummary>,
    breadcrumbs: Vec<BreadcrumbItem>,
    organization: Option<OrganizationInfo>,
    graph_summary: Vec<String>,
    data_downloads: Vec<DataDownloadEntry>,
}
```

### Data Flow

```
URL/JSON-LD Input
    ↓
parser::fetch_html() or direct input
    ↓
parser::extract_json_ld_blocks()
    ↓
parser::combine_json_ld_blocks()
    ↓
ld_graph::expand_json_ld()
    ↓
ld_graph::GraphBuilder::ingest_document()
    ↓
ld_graph::GraphBuilder::into_graph()
    ↓
GraphInsights::from() - Analysis
    ↓
render_*() functions - Output
```

## CLI Interface

### Output Modes

1. **Default** (no flags)
   - Markdown representation of page
   - Product summaries with variant tables
   - Organization info, breadcrumbs
   
2. **`-G, --graph-summary`** (SummaryOnly mode)
   - Product summaries only
   - NO markdown output
   - Useful for structured data extraction

3. **`-g, --graph-only`** (GraphOnly mode)
   - Condensed graph summary only
   - Shows entity types and relationships
   - NO markdown, NO detailed summaries

### All CLI Flags

```
-g, --graph-only        Condensed graph summary only
-G, --graph-summary     Product summaries only (no markdown)
-m, --mermaid           Include Mermaid diagram visualization
-dd, --data-downloads   Include DataDownload references
-s, --save [PATH]       Save markdown output to file
-v, --version           Show version
-h, --help              Show help
```

### Input Modes

**URL Input**:
```bash
htmlens https://example.com/product-page
```

**Direct JSON-LD Input**:
```bash
htmlens '{"@context": "https://schema.org", "@type": "Product", ...}'
```
- Must start with `{` or `[`
- No HTTP request made
- Useful for offline analysis

## Key Functions Reference

### Entry Points
- `main()` - Parses CLI args, routes to help/version/run
- `run(options: CliOptions) -> Result<()>` - Main execution flow
- `parse_arguments(args: &[String]) -> Result<CliCommand>` - CLI argument parser

### Graph Analysis (in main.rs)
- `GraphInsights::from(graph: &KnowledgeGraph) -> GraphInsights` - Extracts product/variant insights
- `summarize_variant(product, adjacency, nodes, ...) -> VariantSummary` - Analyzes product variants
- `extract_offer(product, adjacency, nodes) -> Option<OfferInfo>` - Finds pricing/availability
- `extract_common_properties(product_group, variants) -> HashMap<String, String>` - Identifies shared properties using token-based matching
- `collect_additional_properties(product, adjacency, nodes) -> HashMap<String, String>` - Extracts PropertyValue entities

### Output Rendering (in main.rs)
- `render_variant_table(buf, variants, total)` - Creates markdown table
- `render_graph_summary(buf, lines)` - Outputs condensed relationships
- `render_data_downloads_section(buf, entries)` - Lists DataDownload resources
- `graph_to_mermaid(graph) -> String` - Generates Mermaid diagram

### Utility Functions (in main.rs)
- `property_text(node, keys) -> Option<String>` - Extracts text from node properties (handles multiple key variations)
- `has_schema_type(node, type_name) -> bool` - Checks if node has specific @type
- `shorten_iri(iri) -> String` - Converts full IRI to short name (e.g., "https://schema.org/name" → "name")
- `format_price(value, currency) -> String` - Formats prices with symbols
- `build_output_path(base, url) -> PathBuf` - Generates filename from URL
- `build_adjacency(graph) -> HashMap<String, Vec<&GraphEdge>>` - Creates efficient edge lookup
- `predicate_matches(predicate, name) -> bool` - Flexible predicate matching

## Common Patterns

### Property Resolution with Fallbacks

Properties are checked with multiple key variations (http/https, full IRI, short name):

```rust
property_text(node, &[
    "https://schema.org/name",
    "http://schema.org/name",
    "name"
])
```

This handles:
- Different protocol schemes (http vs https)
- Full IRIs vs short names
- Legacy data inconsistencies

### Schema.org Type Checking

Types are compared case-insensitively after shortening:

```rust
has_schema_type(node, "Product")  
// Matches: "https://schema.org/Product", "http://schema.org/Product", "Product"
```

### Edge Traversal with Adjacency Map

Build adjacency map once for efficient traversal:

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

### Token-Based Property Matching

Avoids false positives in property name matching:

```rust
fn normalize_tokens(name: &str) -> Vec<String> {
    // "FrameSize" → ["frame", "size"]
    // "colorway" → ["colorway"]
}

fn is_varying(prop_name: &str, varies_by: &[String]) -> bool {
    let prop_tokens = normalize_tokens(prop_name);
    // Checks for exact sequence matches
}
```

This prevents "colorway" from matching "color".

### Handling JSON-LD Arrays

The parser handles both object and array-based JSON-LD:

```rust
// Object: {"@context": ..., "@type": "Product", ...}
// Array: [{"@type": "Product"}, {"@type": "Offer"}]
```

When merging multiple blocks, the first `@context` is hoisted to the top level.

## Development Guidelines

### When Adding Features

#### 1. New CLI Options

**Where to edit**: `src/main.rs`

Steps:
1. Update `OutputMode` enum if adding new modes (around line 20)
2. Modify `parse_arguments()` to handle new flags (around lines 45-150)
3. Update `print_help()` with documentation (around lines 175-195)
4. Add logic in `run()` to use the new option (around lines 195-350)

Example:
```rust
// Add to enum
enum OutputMode {
    Default,
    SummaryOnly,
    GraphOnly,
    NewMode,  // ← Add here
}

// Add to parse_arguments()
if matches!(arg.as_str(), "-n" | "--new-mode") {
    mode = OutputMode::NewMode;
    i += 1;
    continue;
}

// Add to print_help()
println!("  -n, --new-mode          Description of new mode");
```

#### 2. New Schema.org Types

**Where to edit**: `src/main.rs` (GraphInsights section)

Steps:
1. Create summary struct for the new type (around lines 350-600)
2. Add field to `GraphInsights` struct
3. Add detection in `GraphInsights::from()` (around lines 600-750)
4. Create rendering function (around lines 1000-1700)
5. Call renderer from `run()` based on output mode

Example:
```rust
// 1. Create summary struct
#[derive(Debug, Clone)]
struct EventSummary {
    name: Option<String>,
    start_date: Option<String>,
    location: Option<String>,
}

// 2. Add to GraphInsights
struct GraphInsights {
    // ... existing fields
    events: Vec<EventSummary>,
}

// 3. Detect in from()
if has_schema_type(node, "Event") {
    let event = EventSummary {
        name: property_text(node, &["name", "https://schema.org/name"]),
        // ... extract other fields
    };
    insights.events.push(event);
}

// 4. Create renderer
fn render_event_section(buf: &mut String, events: &[EventSummary]) {
    push_section_header(buf, "Events");
    for event in events {
        writeln!(buf, "- **{}**", event.name.as_deref().unwrap_or("Unnamed")).ok();
        // ... render other fields
    }
}

// 5. Call from run()
if !insights.events.is_empty() {
    render_event_section(&mut output, &insights.events);
}
```

#### 3. New Output Formats

**Where to edit**: `src/main.rs` (rendering section)

Steps:
1. Add conversion function (around lines 1000-1300)
2. Add CLI flag for the format
3. Update `run()` to call new formatter based on options

Example:
```rust
// 1. Conversion function
fn graph_to_csv(graph: &KnowledgeGraph) -> String {
    let mut csv = String::from("id,type,properties\n");
    for node in &graph.nodes {
        writeln!(
            &mut csv,
            "{},{},{}",
            node.id,
            node.types.join(";"),
            serde_json::to_string(&node.properties).unwrap()
        ).ok();
    }
    csv
}

// 2. Add flag in parse_arguments()
let mut export_csv = false;
if matches!(arg.as_str(), "--csv") {
    export_csv = true;
    // ...
}

// 3. Use in run()
if options.export_csv {
    let csv = graph_to_csv(&graph);
    writeln!(&mut output, "\n## CSV Export\n```csv\n{}\n```", csv)?;
}
```

#### 4. New Property Extractors

**Where to edit**: `src/main.rs` (analysis section)

Follow the `property_text()` pattern for consistency:

```rust
fn extract_new_property(node: &GraphNode) -> Option<String> {
    property_text(node, &[
        "https://schema.org/propertyName",
        "http://schema.org/propertyName",
        "propertyName"
    ])
}
```

Always check multiple key variations (http/https, full/short).

### When Modifying Modules

#### Modifying `parser.rs`

**Common changes**:
- Adjusting HTML sanitization rules
- Adding new JSON-LD detection patterns
- Changing context merging behavior

**Testing**: Use real-world URLs with various JSON-LD structures.

#### Modifying `ld_graph.rs`

**Common changes**:
- Adjusting how blank nodes are handled
- Modifying property vs. edge classification
- Adding support for new JSON-LD features

**Testing**: Use `json-ld` crate's test suite patterns.

#### Modifying `main.rs`

**Common changes**:
- Adding new entity types
- Improving property extraction
- Enhancing output formatting

**Testing**: Run against e-commerce sites with complex product data.

### Code Style

- **Error Handling**: Use `anyhow::Result` and `.context()` for descriptive errors
- **Async**: Use `tokio::main` for async entry point, `reqwest` for HTTP
- **Formatting**: Use `cargo fmt` before committing
- **Naming**: snake_case for functions/variables, PascalCase for types
- **Documentation**: Add doc comments (`///`) for public functions
- **Imports**: Group by standard library, external crates, internal modules

### Testing Approach

Currently no automated tests. When adding tests:

**Unit Tests**:
- Test property extraction with various key formats
- Test token-based property matching
- Test edge cases in URL parsing and filename generation

**Integration Tests**:
- Mock HTTP requests for fetch testing
- Test JSON-LD expansion with sample documents
- Test end-to-end with known HTML fixtures

**Test URLs**:
Good sites for manual testing:
- https://www.kalkhoff-bikes.com (ProductGroup with variants)
- https://www.gazelle.nl (Complex product hierarchies)
- https://schema.org (Multiple JSON-LD blocks)

## Common Modifications

### Adding a New Product Property

**Example**: Add "Material" to product variants

1. **Update VariantSummary** (in main.rs, around line 400):
```rust
struct VariantSummary {
    // ... existing fields
    material: Option<String>,
}
```

2. **Extract in summarize_variant()** (around line 700):
```rust
let mut summary = VariantSummary {
    // ... existing fields
    material: property_text(
        product,
        &["https://schema.org/material", "material"]
    ),
};
```

3. **Add to table rendering** (around line 1100):
```rust
let headers = ["SKU", "Color", "Size", "Material", "Price", "Availability"];

// In row rendering:
row.push(
    variant.material
        .as_deref()
        .unwrap_or("-")
        .to_string()
);
```

### Supporting Multiple ProductGroups

**Already implemented!** The code handles multiple ProductGroups by:

1. Finding all nodes with type "ProductGroup"
2. Processing each independently
3. Extracting variant relationships via `hasVariant`
4. Separating common properties from varying properties using `variesBy`

See `GraphInsights::from()` around line 650.

### Modifying JSON-LD Context Handling

**Location**: `src/parser.rs`, `combine_json_ld_blocks()`

Current behavior:
- Hoists first `@context` to top level
- Subsequent contexts are discarded
- Trade-off: Simplicity vs. correctness

To change:
```rust
// Option 1: Keep all contexts
let contexts: Vec<_> = blocks.iter()
    .filter_map(|b| b.get("@context"))
    .collect();

// Option 2: Merge contexts
// (Requires more sophisticated logic)
```

### Customizing Output Format

All output is built in the `run()` function using a `String` buffer and `writeln!` macro.

Sections are controlled by boolean flags:
- `include_markdown` - Show page markdown
- `include_summary_sections` - Show detailed product tables
- `include_condensed_summary` - Show entity list
- `include_graph_exports` - Show Mermaid/JSON

To add a new section:

```rust
// In run() function
let include_new_section = matches!(options.mode, OutputMode::Default);

if include_new_section && !insights.new_data.is_empty() {
    push_section_header(&mut output, "New Section");
    render_new_section(&mut output, &insights.new_data);
}
```

## Dependencies

### Core Libraries

| Crate | Version | Purpose |
|-------|---------|---------|
| `reqwest` | 0.11 | HTTP client with async support |
| `scraper` | 0.18 | HTML parsing with CSS selectors |
| `html2md` | 0.2 | HTML to Markdown conversion |
| `json-ld` | 0.17.2 | JSON-LD expansion and processing |
| `serde` / `serde_json` | 1.0 | Serialization/deserialization |
| `tokio` | 1.0 | Async runtime |
| `anyhow` | 1.0 | Error handling with context |
| `url` | 2.0 | URL parsing and validation |
| `once_cell` | 1.19 | Lazy static initialization |
| `regex` | 1.10 | Pattern matching |
| `uuid` | 1.11 | Blank node ID generation |

### Version Constraints

- **Rust Edition**: 2024
- **Minimum Rust Version**: 1.85
- See `Cargo.toml` for exact dependency versions

## Troubleshooting

### Common Issues

#### 1. JSON-LD Expansion Fails

**Symptoms**: Error during `expand_json_ld()`

**Causes**:
- Invalid base URL (not a valid IRI)
- Remote contexts are inaccessible (network issues)
- Malformed JSON in script tags

**Solutions**:
- Verify base URL with `url::Url::parse()`
- Check network connectivity for remote contexts
- Validate JSON with `serde_json::from_str()` before expansion

#### 2. Property Not Extracted

**Symptoms**: Expected property shows as "-" in output

**Causes**:
- Property key mismatch (http vs https)
- Property is a node reference, not literal
- Property exists but is array/object

**Solutions**:
- Add more key variations to `property_text()` call
- Use `resolve_node_property()` for node references
- Handle arrays with `.as_array()` and extract first element

#### 3. Graph Relationships Missing

**Symptoms**: Mermaid diagram incomplete, edges not shown

**Causes**:
- Node IDs don't match between edges and nodes
- Predicate matching too strict
- JSON-LD expansion didn't include relationships

**Solutions**:
- Debug print node IDs during graph building
- Use `predicate_matches()` for flexible matching
- Check expanded JSON-LD output

#### 4. HTML Sanitization Too Aggressive

**Symptoms**: Important content removed from markdown

**Causes**:
- Regex patterns too broad in `sanitize_html()`

**Solutions**:
- Adjust regex patterns in `src/parser.rs`
- Test with specific HTML structures
- Consider using CSS selectors instead

#### 5. Merge Conflicts When Combining JSON-LD Blocks

**Symptoms**: Properties lost when combining multiple blocks

**Causes**:
- Context hoisting discards later contexts
- Duplicate keys in merged objects

**Solutions**:
- Review `combine_json_ld_blocks()` logic
- Consider keeping separate graphs
- Document trade-offs in comments

## Performance Considerations

### Current Performance Profile

- **Fast**: Single page processing (~1-2 seconds)
- **Bottlenecks**: 
  - Network requests (HTML fetch + remote contexts)
  - JSON-LD expansion (can be slow for large documents)
  - Markdown conversion for large pages

### Optimization Opportunities

#### 1. Parallel Processing
```rust
// Process multiple JSON-LD blocks in parallel
use tokio::task::spawn;

let handles: Vec<_> = blocks.into_iter()
    .map(|block| spawn(expand_json_ld(base_url, &block, loader)))
    .collect();

let results = join_all(handles).await;
```

#### 2. Context Caching
```rust
// Cache remote contexts to avoid repeated downloads
use once_cell::sync::Lazy;
use std::collections::HashMap;

static CONTEXT_CACHE: Lazy<Mutex<HashMap<String, Value>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));
```

#### 3. Streaming Output
```rust
// Stream output instead of building in memory
use std::io::{self, Write};

fn render_streaming(graph: &KnowledgeGraph, writer: &mut dyn Write) {
    for node in &graph.nodes {
        writeln!(writer, "{:?}", node)?;
    }
}
```

#### 4. Incremental Parsing
For very large HTML documents, consider using a streaming HTML parser instead of loading entire document into memory.

## Future Enhancement Ideas

### Refactoring Opportunities

- **Add unit tests** for core functions (property extraction, token matching)
- **Extract render module** - Move all rendering functions to `src/render.rs`
- **Add config file support** - YAML/TOML for default options
- **Create trait abstractions** - `EntityExtractor` trait for different Schema.org types

### Feature Ideas

#### Short-term
- **Batch processing**: Process multiple URLs from file
- **JSON/CSV output**: Machine-readable export formats
- **Filter by entity type**: Only extract specific Schema.org types
- **Verbosity levels**: Control output detail with `-v`, `-vv`, `-vvv`

#### Medium-term
- **Interactive mode**: Explore graph with queries
- **Diff mode**: Compare two pages for changes
- **Watch mode**: Monitor URL for updates
- **OpenGraph/Twitter Cards**: Extract social media metadata

#### Long-term
- **Plugin system**: Load custom extractors as dynamic libraries
- **Web UI**: Browser-based interface
- **API mode**: Run as HTTP service
- **Database storage**: Store graphs in SQLite/PostgreSQL

### Schema.org Type Coverage

Currently supported:
- ✅ Product / ProductGroup
- ✅ Organization
- ✅ BreadcrumbList
- ✅ Offer
- ✅ PropertyValue
- ✅ DataDownload

Consider adding:
- ❌ Article / BlogPosting
- ❌ Event
- ❌ Recipe
- ❌ Place / LocalBusiness
- ❌ Review / AggregateRating
- ❌ Person
- ❌ VideoObject / ImageObject

## Getting Help

### Resources

1. **User Documentation**: See `README.md` for usage examples
2. **Rust Docs**: Run `cargo doc --open` for dependency documentation
3. **Schema.org**: https://schema.org for entity definitions
4. **JSON-LD**: https://json-ld.org for specification

### Debugging Tips

**Enable debug output**:
```rust
// Add debug prints to trace execution
eprintln!("DEBUG: Processing node: {:?}", node.id);
```

**Inspect intermediate results**:
```bash
# Save expanded JSON-LD to file
htmlens https://example.com > output.txt

# Check just the graph
htmlens https://example.com -g

# Get raw JSON-LD blocks
# (requires adding debug output)
```

**Use cargo features**:
```bash
cargo build --release          # Optimized build
cargo clippy                   # Linting
cargo fmt                      # Format code
cargo test                     # Run tests (when added)
```

## Contributing Guidelines

### Before Making Changes

1. **Understand the data flow**: Follow data from input → parser → graph → output
2. **Read existing code**: Look for similar patterns before adding new code
3. **Check for duplication**: Reuse existing functions when possible
4. **Consider module boundaries**: Put code in the right module

### Making Changes

1. **Create a feature branch**: `git checkout -b feature/description`
2. **Make focused commits**: One logical change per commit
3. **Write clear commit messages**: Explain why, not just what
4. **Test manually**: Use real-world URLs to verify changes
5. **Run code quality checks**: `cargo fmt && cargo clippy`

### Submitting Changes

1. **Update README.md** if adding user-facing features
2. **Update AGENTS.md** if changing architecture or adding modules
3. **Provide test URLs** in PR description
4. **Explain trade-offs** if making design decisions
5. **Keep PRs focused**: One feature or fix per PR

### Code Review Checklist

- [ ] Code follows Rust idioms and conventions
- [ ] Error handling is comprehensive with good context
- [ ] Functions are documented with `///` comments
- [ ] No unwrap() or expect() in production paths
- [ ] Module boundaries are respected
- [ ] Performance impact is considered
- [ ] Backward compatibility is maintained

## Practical Examples

### Example 1: Adding Support for Recipe Schema

**Goal**: Extract and display recipe information

**Steps**:

1. **Add data structure** (main.rs ~line 450):
```rust
#[derive(Debug, Clone)]
struct RecipeSummary {
    name: Option<String>,
    description: Option<String>,
    prep_time: Option<String>,
    cook_time: Option<String>,
    ingredients: Vec<String>,
    instructions: Vec<String>,
}
```

2. **Add to GraphInsights** (~line 380):
```rust
struct GraphInsights {
    // ... existing
    recipes: Vec<RecipeSummary>,
}
```

3. **Extract in from()** (~line 700):
```rust
// Look for Recipe nodes
for node in nodes_map.values() {
    if has_schema_type(node, "Recipe") {
        let recipe = RecipeSummary {
            name: property_text(node, &["name"]),
            description: property_text(node, &["description"]),
            prep_time: property_text(node, &["prepTime"]),
            cook_time: property_text(node, &["cookTime"]),
            ingredients: extract_text_array(node, &["recipeIngredient"]),
            instructions: extract_text_array(node, &["recipeInstructions"]),
        };
        insights.recipes.push(recipe);
    }
}
```

4. **Add helper for arrays** (~line 1750):
```rust
fn extract_text_array(node: &GraphNode, keys: &[&str]) -> Vec<String> {
    for key in keys {
        if let Some(value) = node.properties.get(*key) {
            if let Some(arr) = value.as_array() {
                return arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }
    }
    Vec::new()
}
```

5. **Add renderer** (~line 1300):
```rust
fn render_recipe_section(buf: &mut String, recipes: &[RecipeSummary]) {
    for recipe in recipes {
        push_section_header(buf, "Recipe");
        if let Some(name) = &recipe.name {
            writeln!(buf, "**{}**\n", name).ok();
        }
        if let Some(desc) = &recipe.description {
            writeln!(buf, "{}\n", desc).ok();
        }
        if !recipe.ingredients.is_empty() {
            writeln!(buf, "**Ingredients:**").ok();
            for ing in &recipe.ingredients {
                writeln!(buf, "- {}", ing).ok();
            }
            writeln!(buf).ok();
        }
        if !recipe.instructions.is_empty() {
            writeln!(buf, "**Instructions:**").ok();
            for (i, step) in recipe.instructions.iter().enumerate() {
                writeln!(buf, "{}. {}", i + 1, step).ok();
            }
            writeln!(buf).ok();
        }
    }
}
```

6. **Call from run()** (~line 300):
```rust
if !insights.recipes.is_empty() {
    render_recipe_section(&mut output, &insights.recipes);
}
```

### Example 2: Adding CSV Export

**Goal**: Export graph data as CSV

1. **Add CLI flag** (main.rs ~line 55):
```rust
let mut export_csv = false;

// In parse loop
if matches!(arg.as_str(), "--csv") {
    export_csv = true;
    i += 1;
    continue;
}
```

2. **Add to CliOptions** (~line 35):
```rust
struct CliOptions {
    // ... existing
    export_csv: bool,
}
```

3. **Add converter** (~line 1600):
```rust
fn graph_to_csv(graph: &KnowledgeGraph) -> String {
    let mut csv = String::from("id,types,property_count\n");
    for node in &graph.nodes {
        csv.push_str(&format!(
            "\"{}\",\"{}\",{}\n",
            node.id,
            node.types.join(";"),
            node.properties.len()
        ));
    }
    csv
}
```

4. **Use in run()** (~line 330):
```rust
if options.export_csv {
    writeln!(&mut output, "\n## CSV Export\n").ok();
    writeln!(&mut output, "```csv").ok();
    writeln!(&mut output, "{}", graph_to_csv(&graph)).ok();
    writeln!(&mut output, "```").ok();
}
```

5. **Update help** (~line 185):
```rust
println!("  --csv                   Export graph as CSV");
```

---

**Remember**: This tool helps marketers and developers understand how search engines see their content. Keep the output human-readable, the code maintainable, and the architecture modular.

**Questions?** Check the code comments, experiment with test URLs, and don't hesitate to add debug output to understand the data flow.
