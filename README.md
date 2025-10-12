# HTML Lens

See the semantic web through a clear lens.

`htmlens` reveals the structured reality hiding inside raw HTML by expanding
JSON‑LD, mapping Schema.org entities, and presenting the inferred knowledge
graph alongside the source content.

- A Markdown representation of the page.
- A JSON serialization of extracted graph nodes and relationships.
- A Mermaid diagram describing entity connections.
- A list of any `DataDownload` `contentUrl` values discovered in the graph.

## Prerequisites

- Rust 1.70 or newer (stable toolchain recommended).
- Network access from the environment where the binary runs.

## Build

```bash
cargo build
```

## Run

```bash
htmlens --help
```

```bash
htmlens [OPTIONS] <url>
```

Example:

```bash
htmlens https://example.com/product-page --graph-summary --save reports
```

Running directly with Cargo:

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

## Implementation Notes

- Uses `reqwest` for HTTP, `html2md` to generate Markdown, and `scraper` to
  locate `application/ld+json` blocks.
- JSON‑LD expansion relies on the `json-ld` crate’s `ReqwestLoader` to resolve
  remote contexts.
- The graph builder normalizes node identifiers, collects literal properties,
  and tracks edges (`offers`, `brand`, `hasVariant`, etc.) between nodes.
- `DataDownload` entities are detected by scanning the expanded document and
  collecting `contentUrl` values regardless of compacted or expanded keys.
