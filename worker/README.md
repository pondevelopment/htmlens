# Htmlens Cloudflare Worker

A Cloudflare Worker that provides an API for extracting semantic knowledge graphs from HTML pages using the htmlens WASM module.

## Status

✅ **Worker Infrastructure**: Complete and running
⏳ **WASM Integration**: In progress (building htmlens WASM module)

## Architecture

- **Worker Runtime**: Cloudflare Workers (Edge Computing)
- **Language**: TypeScript
- **WASM Module**: htmlens (Rust compiled to WebAssembly)
- **API**: REST-style GET endpoint

## Setup

### Prerequisites

- Node.js 20+ (managed via NVM)
- Wrangler CLI (installed via npm)
- htmlens WASM module (from parent `../pkg/` directory)

### Installation

```bash
npm install
```

### Development

Run the Worker locally:

```bash
npm run dev
```

The Worker will be available at `http://localhost:8787`

### Deployment

Deploy to Cloudflare:

```bash
npm run deploy
```

## API Usage

### Get API Info

```bash
curl http://localhost:8787/
```

### Process a URL (Once WASM is integrated)

```bash
curl "http://localhost:8787/?url=https://example.com"
```

### Health Check

```bash
curl http://localhost:8787/health
```

## Response Format

```json
{
  "url": "https://example.com",
  "markdown": "# Converted markdown content...",
  "json_ld_blocks": ["..."],
  "knowledge_graph": {
    "nodes": [...],
    "edges": [...]
  }
}
```

## Features

### Current (v0.2.0)
- ✅ Worker infrastructure and routing
- ✅ CORS support
- ✅ Health check endpoint
- ✅ API documentation endpoint
- ✅ Error handling

### Planned (once WASM is integrated)
- ⏳ HTML to Markdown conversion
- ⏳ JSON-LD block extraction
- ⏳ Knowledge graph generation
- ⏳ Mermaid diagram generation
- ⏳ Product/variant insights

## Project Structure

```
worker/
├── src/
│   └── index.ts          # Main Worker code
├── package.json          # Node.js dependencies
├── tsconfig.json         # TypeScript configuration
├── wrangler.toml         # Cloudflare Worker configuration
└── README.md             # This file
```

## Configuration

### wrangler.toml

The Worker configuration is defined in `wrangler.toml`:

```toml
name = "htmlens-worker"
main = "src/index.ts"
compatibility_date = "2024-10-01"

# WASM module will be added once built:
# [wasm_modules]
# HTMLENS = "../pkg/htmlens_bg.wasm"
```

## Integration with WASM

Once the htmlens WASM module is built:

1. The WASM file will be available at `../pkg/htmlens_bg.wasm`
2. Update `wrangler.toml` to include the WASM module
3. Import and use the WASM functions in `src/index.ts`
4. Process HTML and return results

Example WASM usage (once integrated):

```typescript
import * as htmlens from '../pkg/htmlens';

// Extract JSON-LD blocks
const jsonLdBlocks = htmlens.extract_json_ld_blocks_wasm(html);

// Convert HTML to Markdown
const markdown = htmlens.html_to_markdown_wasm(html);

// Generate Mermaid diagram from graph
const diagram = htmlens.graph_to_mermaid_wasm(graph);
```

## Development Notes

### WASM Build Configuration

The parent Rust project requires special configuration for WASM:

1. `.cargo/config.toml` with getrandom backend configuration
2. Feature flags: `wasm` feature enables WASM-specific dependencies
3. Build command: `wasm-pack build --target web --no-default-features --features wasm`

### Limitations

- JSON-LD expansion requires server-side processing (json-ld crate doesn't support WASM)
- Workers can extract JSON-LD blocks and send them to a server for full processing
- Core functionality (HTML parsing, markdown conversion, graph visualization) works in WASM

## Testing

Test the API endpoints:

```bash
# API info
curl http://localhost:8787/

# Health check
curl http://localhost:8787/health

# Process URL (once WASM is integrated)
curl "http://localhost:8787/?url=https://schema.org"
```

## Contributing

This is part of the htmlens project. See the main README for contribution guidelines.

## License

MIT - See LICENSE file in the parent directory
