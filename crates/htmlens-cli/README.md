# htmlens-cli

Command-line tool for extracting semantic knowledge graphs from HTML pages.

## Installation

```bash
cargo install htmlens-cli
```

Or from source:

```bash
cd crates/htmlens-cli
cargo build --release
```

## Usage

### Extract from URL

```bash
htmlens https://www.kalkhoff-bikes.com/de_de/entice-7-advance
```

### Direct JSON-LD Input

```bash
htmlens '{"@context": "https://schema.org", "@type": "Product", "name": "Example"}'
```

### Output Modes

**Default**: Markdown + product summaries
```bash
htmlens https://example.com
```

**Summary Only** (`-G`): Product summaries without markdown
```bash
htmlens https://example.com -G
```

**Graph Only** (`-g`): Condensed entity summary
```bash
htmlens https://example.com -g
```

### Additional Options

- `-m, --mermaid`: Include Mermaid diagram visualization
- `-dd, --data-downloads`: Include DataDownload references
- `-s, --save [PATH]`: Save output to file
- `-v, --version`: Show version
- `-h, --help`: Show help

### Examples

```bash
# Extract product variants with Mermaid diagram
htmlens https://www.focus-bikes.com/nl_nl/paralane-8-8 -G -m

# Save full analysis to file
htmlens https://www.gazelle.nl/fietsen/tour-populair-c8 -s output.md

# Quick entity overview
htmlens https://schema.org -g
```

## Features

- ✅ Extract JSON-LD from HTML pages
- ✅ Website WebMCP support scan (multi-signal + active probes)
- ✅ Analyze ProductGroup with variants
- ✅ Extract offers, pricing, and availability
- ✅ Build breadcrumb navigation
- ✅ Identify organization information
- ✅ Generate Mermaid diagrams
- ✅ Support all standard Schema.org Product properties
- ✅ Dynamic variant table columns based on `variesBy`
- ✅ Markdown conversion of page content

### WebMCP scan output

When the input is a URL, htmlens now includes a **WebMCP Scan** section with:
- support status (`supported`)
- confidence (`low`/`medium`/`high`) and score (`0-100`)
- detected signals
- outdated metadata findings (legacy links / old protocol versions)
- all available discovered routes
- all detected available tools (from `registerTool(...)` and MCP metadata)
- active probe results for well-known files and discovered endpoint
- in-page WebMCP API signals (e.g. `document.modelContext` / `registerTool`)

In `--graph-only` mode, it also adds a condensed single-line WebMCP status.

## License

MIT
