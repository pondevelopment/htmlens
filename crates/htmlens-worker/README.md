# htmlens-worker

Cloudflare Worker for lightweight JSON-LD extraction from HTML pages.

## Overview

This is a lightweight API endpoint for extracting JSON-LD blocks from HTML. It uses `htmlens-core` **without** the `full-expansion` feature to keep the WASM bundle small and fast.

## Limitations

- ❌ **No JSON-LD Expansion**: Does not resolve remote `@context` or expand shorthand notation
- ❌ **No Graph Building**: Returns raw JSON-LD blocks only
- ✅ **Fast**: Minimal dependencies, optimized for V8 isolate
- ✅ **Simple**: Extract JSON-LD blocks from HTML or URLs

## API Endpoints

### `POST /extract`

Extract JSON-LD blocks from a URL or HTML content.

**Request (from URL)**:
```json
{
  "url": "https://www.kalkhoff-bikes.com/de_de/entice-7-advance"
}
```

**Request (from HTML)**:
```json
{
  "html": "<script type=\"application/ld+json\">{...}</script>"
}
```

**Response**:
```json
{
  "success": true,
  "json_ld": [
    {"@context": "https://schema.org", "@type": "Product", ...},
    {"@context": "https://schema.org", "@type": "Organization", ...}
  ]
}
```

**Error Response**:
```json
{
  "success": false,
  "json_ld": [],
  "error": "Extraction failed: Invalid HTML"
}
```

### `GET /health`

Health check endpoint.

**Response**:
```json
{
  "status": "ok",
  "version": "0.4.0"
}
```

## Development

### Prerequisites

- Rust 1.85+
- [wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/)
- Cloudflare account (for deployment)

### Local Development

```bash
cd crates/htmlens-worker
wrangler dev
```

Test locally:
```bash
curl -X POST http://localhost:8787/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.kalkhoff-bikes.com/de_de/entice-7-advance"}'
```

### Deployment

1. Configure `wrangler.toml` with your route:
```toml
[[routes]]
pattern = "htmlens-api.example.com/*"
zone_name = "example.com"
```

2. Deploy:
```bash
wrangler deploy
```

## Architecture

- **Lightweight**: No JSON-LD expansion = smaller WASM bundle (~2MB vs ~10MB)
- **Fast Startup**: No heavy dependencies = faster cold starts (~5ms vs ~50ms)
- **Simple API**: Extract only, analysis happens client-side or via CLI

## Two-Tier Strategy

Use **Worker** for fast extraction, **CLI** for deep analysis:

1. **Worker**: Extract JSON-LD from HTML → return to client
2. **Client/CLI**: Full JSON-LD expansion and graph building

This provides:
- ⚡ Fast API responses (Worker)
- 🔬 Deep analysis when needed (CLI)
- 🔄 Code reuse via `htmlens-core`

## License

MIT
