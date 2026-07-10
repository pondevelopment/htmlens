# Project Organization ("Setup")

HTMLens is implemented as a Rust **Cargo workspace** (`Cargo.toml`, edition 2024, current version `0.5.1`). Shared package metadata and dependencies are defined once under `[workspace.package]` and `[workspace.dependencies]` and inherited by each crate.

> For deep architectural detail, module responsibilities, data flow, and step-by-step extension recipes, see [AGENTS.md](../AGENTS.md). Keep that file in sync when the architecture changes.

## Workspace Layout

```
htmlens/
├── Cargo.toml            # Workspace definition + shared deps
├── Makefile              # Common build/test/install targets
├── crates/
│   ├── htmlens-core/     # 🔧 Core library (parsing, graph, AI readiness)
│   ├── htmlens-cli/      # 📦 Command-line tool
│   └── htmlens-worker/   # ☁️ Cloudflare Worker (WASM web API + UI)
├── reports/              # Example output files
└── tests/fixtures/       # HTML/JSON-LD test fixtures
```

## Crates and Responsibilities

- **`htmlens-core`** — Reusable library shared by CLI and Worker. Modules:
  - `types.rs` — Core data structures (`KnowledgeGraph`, `GraphNode`, `GraphEdge`); always compiled.
  - `parser.rs` — HTML fetch/sanitize, JSON-LD extraction and combining, HTML→Markdown; always compiled.
  - `url_utils.rs` — URL/origin normalization helpers.
  - `graph.rs` — JSON-LD expansion + knowledge graph construction; feature-gated.
  - `ai_readiness/` — AI/AEO signal checks (`well_known`, `robots_txt`, `sitemap`, `openapi`, `mcp_manifest`, `plugin_manifest`, `semantic_html`); feature-gated. Produces `AiReadinessReport` with score, strengths, issues, and recommendations.
- **`htmlens-cli`** — Full-featured command-line tool. Enables heavy features, performs entity extraction and formatted rendering (tables, summaries, Mermaid).
- **`htmlens-worker`** — Lightweight Cloudflare Worker compiled to `wasm32-unknown-unknown`. Serves the web UI (`frontend.html`) and a JSON API. Avoids heavy expansion deps for fast cold starts.

## Feature Flags (defined in `htmlens-core`)

- `default` (empty) — Lightweight parsing only; WASM/worker friendly, no heavy deps.
- `full-expansion` — Adds JSON-LD expansion + `graph` module (`json-ld`, `reqwest`, `tokio`, `uuid`). Used by the CLI.
- `ai-readiness` — AI readiness checks with HTTP client (`reqwest`, `tokio`, `openapiv3`, `serde_yaml`).
- `ai-readiness-parser` — WASM-compatible AI readiness parsing only (no HTTP).

Rule of thumb: keep the Worker on lightweight/WASM-safe features; the CLI opts into the heavy features.

## Build & Test (via `Makefile`)

- `make build` / `make build-release` — Build all crates.
- `make cli` — Build the CLI (`-p htmlens-cli`, release).
- `make worker` — Build the Worker (`-p htmlens-worker`, release).
- `make test` — Run `cargo test --workspace`.
- `make check` — `cargo check --workspace` without building.
- `make install` — Install the CLI globally from `crates/htmlens-cli`.

Always run `cargo fmt` and `cargo clippy` before committing.

## Where to Make Changes

- Shared parsing/graph/AI-readiness logic → `crates/htmlens-core/`.
- CLI flags, entity analysis, and output rendering → `crates/htmlens-cli/src/main.rs`.
- Web API endpoints or UI → `crates/htmlens-worker/src/lib.rs` and `frontend.html`.
- New workspace-wide dependency → add under `[workspace.dependencies]` and reference with `.workspace = true`.

---

# HTMLens Product Strategy

## Vision
HTMLens helps organizations understand and improve how visible, understandable, and usable their websites are for AI systems and answer engines.

## Mission
Turn complex technical web signals into clear strategic insight and prioritized actions for AEO and AI readiness.

## Core Problem
Most teams do not know whether AI systems can reliably discover, interpret, and use their content and services. This creates hidden visibility gaps and missed growth opportunities.

## Product Promise
From one URL, HTMLens delivers a clear overview of AI/AEO readiness and the most important actions to improve it.

## Strategic Pillars
1. **Discoverability**  
   Ensure sites can be found and crawled by search engines and AI agents.

2. **Comprehension**  
   Ensure content is machine-readable, structured, and semantically clear.

3. **Interoperability**  
   Ensure sites and APIs can be consumed safely by modern AI agents and tools.

4. **Trust**  
   Ensure governance, security, and policy signals are visible and clear.

## Target Users
- SEO and AEO teams
- Technical marketing teams
- Product and engineering teams
- Agencies and consultants

## Product Outcome
HTMLens should answer, with confidence:
- How ready is this site for AI and answer engines today?
- What blocks progress most?
- What should be fixed first for the highest impact?

## North Star
**From URL to AI/AEO action plan in seconds.**

## Success Criteria
- Teams can understand readiness status immediately.
- Recommended actions are clear and prioritized.
- Repeated scans show measurable readiness improvement over time.
