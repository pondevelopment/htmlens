# AI Readiness Feature Plan

**Branch**: `ai-features`  
**Started**: October 21, 2025  
**Goal**: Extend htmlens to check and validate website AI-readiness capabilities

Based on: `ai-agents-website.pdf` - "Communicating Website Capabilities to AI Agents"

---

## Overview

Currently, htmlens extracts and analyzes Schema.org JSON-LD structured data. This plan extends it to become a comprehensive **AI-readiness checker** that validates all the ways websites communicate with AI agents.

---

## Phase 1: Core AI Capability Detection 🔍

### 1.1 `.well-known/` Directory Checks
**Status**: ⏳ Not Started

**Scope**: Check for standard AI-related files in `/.well-known/` directory

**Implementation**:
- [ ] Add HTTP client function to check `.well-known/` URLs
- [ ] Check for `/.well-known/ai-plugin.json` (ChatGPT plugins)
- [ ] Check for `/.well-known/openid-configuration` (OAuth/OIDC)
- [ ] Check for `/.well-known/security.txt` (Security policy)
- [ ] Check for `/.well-known/apple-app-site-association` (iOS app links)
- [ ] Check for `/.well-known/assetlinks.json` (Android app links)
- [ ] Report HTTP status codes (200 = present, 404 = absent)
- [ ] Validate JSON format for JSON files
- [ ] Validate text format for security.txt

**Output**:
```markdown
## AI Integration Files

### .well-known Directory
- ✅ `/.well-known/ai-plugin.json` - Found (ChatGPT plugin)
- ❌ `/.well-known/openid-configuration` - Not found
- ✅ `/.well-known/security.txt` - Found
```

**Priority**: 🔴 **Critical** - Core AI capability detection

**Estimated Effort**: 2-3 hours

---

### 1.2 AI Plugin Manifest Validation
**Status**: ⏳ Not Started

**Scope**: Parse and validate `/.well-known/ai-plugin.json` structure

**Implementation**:
- [ ] Create Rust struct for AI Plugin Manifest schema (v1)
- [ ] Parse manifest JSON and validate required fields:
  - [ ] `schema_version`
  - [ ] `name_for_human` / `name_for_model`
  - [ ] `description_for_human` / `description_for_model`
  - [ ] `auth` (type: none/user_http/service_http)
  - [ ] `api` (type, url, is_user_authenticated)
  - [ ] `logo_url`, `contact_email`, `legal_info_url`
- [ ] Validate field formats:
  - [ ] URLs are valid and reachable
  - [ ] Email is valid format
  - [ ] `name_for_model` has no spaces
  - [ ] `description_for_model` is under 8000 chars
- [ ] Fetch and validate OpenAPI spec URL from `api.url`
- [ ] Check auth type matches between manifest and OpenAPI spec
- [ ] Generate detailed report with recommendations

**Output**:
```markdown
## AI Plugin Manifest

### Status: ✅ Valid
- **Name**: TODO Plugin
- **Model Name**: todo
- **Auth**: None
- **API Spec**: https://example.com/openapi.yaml ✅

### Validation Results:
- ✅ All required fields present
- ✅ URLs accessible
- ✅ Email format valid
- ✅ OpenAPI spec validated
- ⚠️ Description for model is short (245 chars) - consider adding more detail
```

**Priority**: 🔴 **Critical** - Validates AI plugin integration

**Estimated Effort**: 4-5 hours

---

### 1.3 OpenAPI/Swagger Specification Validation
**Status**: ⏳ Not Started

**Scope**: Fetch, parse, and validate OpenAPI specifications

**Implementation**:
- [ ] Add OpenAPI parser (use `openapiv3` crate)
- [ ] Fetch OpenAPI spec from URL (JSON or YAML)
- [ ] Validate against OpenAPI 3.x schema
- [ ] Check for required sections:
  - [ ] `openapi` version
  - [ ] `info` (title, version, description)
  - [ ] `paths` with endpoints
  - [ ] `servers` with base URL
  - [ ] `components/schemas` for data models
  - [ ] `securitySchemes` if auth required
- [ ] Verify completeness:
  - [ ] All paths have responses defined
  - [ ] 200 OK responses exist
  - [ ] Parameters have schemas
  - [ ] Security schemes match manifest auth type
- [ ] Generate summary of available endpoints

**Output**:
```markdown
## API Specification

### OpenAPI 3.0.1 - Example API v1.0
**Base URL**: https://api.example.com

### Endpoints Found:
- `GET /items` - List all items
- `POST /items` - Create new item
- `GET /items/{id}` - Get item by ID
- `DELETE /items/{id}` - Delete item

### Validation:
- ✅ Valid OpenAPI 3.0.1 format
- ✅ All endpoints have response schemas
- ✅ Security scheme defined (Bearer token)
- ⚠️ Missing example responses for some endpoints
```

**Priority**: 🟡 **High** - Important for API integrations

**Estimated Effort**: 5-6 hours

---

## Phase 2: Content Discovery & Crawling 🕷️

### 2.1 Robots.txt Parser
**Status**: ⏳ Not Started

**Scope**: Fetch and parse robots.txt crawling rules

**Implementation**:
- [ ] Fetch `/robots.txt`
- [ ] Parse robots.txt format (User-agent, Disallow, Allow, Sitemap)
- [ ] Extract rules for different user-agents:
  - [ ] Wildcard (`*`)
  - [ ] Googlebot
  - [ ] GPTBot / ChatGPT-User (AI crawlers)
  - [ ] Other AI bots (ClaudeBot, Bingbot, etc.)
- [ ] Identify disallowed paths
- [ ] Extract Sitemap URLs
- [ ] Check for overly restrictive rules (e.g., `Disallow: /` for all agents)
- [ ] Flag potential issues:
  - [ ] Blocking all bots accidentally
  - [ ] Missing sitemap reference
  - [ ] Syntax errors

**Output**:
```markdown
## Robots.txt Analysis

### Status: ✅ Found

### Crawling Rules:
**All Bots (`*`)**:
- ❌ Disallowed: `/admin/`, `/private/`
- ✅ Allowed: `/` (all other paths)

**Googlebot**:
- ✅ Full access

**GPTBot (OpenAI)**:
- ⚠️ Blocked from entire site (`Disallow: /`)

### Sitemaps Referenced:
- https://example.com/sitemap.xml

### Recommendations:
- ⚠️ GPTBot is blocked - consider allowing if you want AI to learn from your content
```

**Priority**: 🟡 **High** - Critical for AI crawler access

**Estimated Effort**: 3-4 hours

---

### 2.2 XML Sitemap Validator
**Status**: ⏳ Not Started

**Scope**: Fetch, parse, and validate XML sitemaps

**Implementation**:
- [ ] Fetch sitemap URL (from robots.txt or default `/sitemap.xml`)
- [ ] Parse XML sitemap format
- [ ] Validate against sitemap schema
- [ ] Extract URL entries with metadata:
  - [ ] `<loc>` - URL
  - [ ] `<lastmod>` - Last modified date
  - [ ] `<changefreq>` - Change frequency
  - [ ] `<priority>` - Priority (0.0-1.0)
- [ ] Handle sitemap index files (multiple sitemaps)
- [ ] Check for issues:
  - [ ] URLs on wrong domain
  - [ ] Invalid date formats
  - [ ] Too many URLs (>50,000 per file)
  - [ ] Unreachable URLs (spot-check sample)
- [ ] Generate statistics and coverage report

**Output**:
```markdown
## XML Sitemap

### Status: ✅ Found at `/sitemap.xml`

### Statistics:
- **Total URLs**: 1,247
- **Last Updated**: 2025-10-15
- **Format**: Valid XML

### URL Distribution:
- Products: 850 URLs
- Blog posts: 320 URLs
- Pages: 77 URLs

### Validation:
- ✅ Valid XML format
- ✅ All URLs on correct domain
- ✅ Valid date formats
- ✅ Under 50,000 URL limit
- ℹ️ Spot-checked 10 URLs - all accessible

### Recommendations:
- ✅ Sitemap properly referenced in robots.txt
```

**Priority**: 🟡 **High** - Important for content discovery

**Estimated Effort**: 4-5 hours

---

## Phase 3: Enhanced Structured Data 📊

### 3.1 Expanded Schema.org Validation
**Status**: ⏳ Not Started (extends existing functionality)

**Scope**: Enhance current JSON-LD extraction with deeper validation

**Implementation**:
- [ ] Build on existing `extract_json_ld_blocks()` function
- [ ] Add validation for common Schema.org types:
  - [x] Product / ProductGroup (already exists)
  - [x] Organization (already exists)
  - [x] BreadcrumbList (already exists)
  - [ ] Article / BlogPosting
  - [ ] FAQ / Question / Answer
  - [ ] Event
  - [ ] Recipe
  - [ ] Review / AggregateRating
  - [ ] LocalBusiness
  - [ ] VideoObject / ImageObject
- [ ] Check for required properties per type (per Google guidelines)
- [ ] Validate against Google Rich Results eligibility
- [ ] Cross-reference with Google's Structured Data Testing Tool criteria
- [ ] Flag missing or invalid properties

**Output**: *(Extends existing insights output)*
```markdown
## Schema.org Structured Data

### Types Found:
- ✅ Product (5 instances)
- ✅ Organization (1 instance)
- ✅ Article (12 instances)
- ✅ BreadcrumbList (1 instance)

### Rich Results Eligibility:
- ✅ **Product Rich Results**: Eligible (all required fields present)
- ⚠️ **Article Rich Results**: Needs work
  - Missing: `datePublished`, `author`
  - Recommended: Add `image`, `headline`
```

**Priority**: 🟢 **Recommended** - Enhances existing feature

**Estimated Effort**: 3-4 hours

---

### 3.2 Structured Data Coverage Report
**Status**: ⏳ Not Started

**Scope**: Analyze which pages have structured data

**Implementation**:
- [ ] Track which types of structured data appear on which pages
- [ ] Calculate coverage percentage
- [ ] Identify pages missing structured data
- [ ] Recommend which pages would benefit most
- [ ] Generate coverage visualization/report

**Output**:
```markdown
## Structured Data Coverage

### Overall Coverage: 78%
- Pages with structured data: 156 / 200
- Pages without structured data: 44

### By Page Type:
- ✅ Product pages: 100% (all 85 pages)
- ⚠️ Blog posts: 65% (45 / 70 pages)
- ❌ About pages: 20% (1 / 5 pages)

### Recommendations:
- Add Article schema to 25 blog posts missing it
- Add Organization schema to About page
- Add FAQ schema to 15 support pages
```

**Priority**: 🟢 **Recommended** - Useful for content strategy

**Estimated Effort**: 3-4 hours

---

## Phase 4: Additional Integrations 🔧

### 4.1 HTTP Header Analysis
**Status**: ⏳ Not Started

**Scope**: Analyze HTTP headers relevant to AI agents

**Implementation**:
- [ ] Fetch HTTP headers for key pages (homepage, API endpoints)
- [ ] Check for relevant headers:
  - [ ] `X-Robots-Tag` (crawler directives)
  - [ ] `Link` with `rel="service-desc"` (API description)
  - [ ] `Link` with `rel="service-doc"` (API documentation)
  - [ ] `Link` with `rel="sitemap"` (sitemap reference)
  - [ ] `Content-Type` (correct MIME types)
  - [ ] `Access-Control-Allow-Origin` (CORS for API access)
- [ ] Validate header values
- [ ] Flag misconfigurations or missing headers
- [ ] Report on discoverability via headers

**Output**:
```markdown
## HTTP Headers Analysis

### Homepage (https://example.com):
- ✅ `Content-Type: text/html; charset=UTF-8`
- ℹ️ `X-Robots-Tag`: Not present (defaults allow indexing)
- ❌ `Link` headers: None found

### API Endpoint (https://api.example.com):
- ✅ `Content-Type: application/json`
- ✅ `Access-Control-Allow-Origin: *` (CORS enabled)
- ✅ `Link: <https://api.example.com/openapi.yaml>; rel="service-desc"`

### Recommendations:
- Consider adding Link header with sitemap reference
- API headers look good for AI agent access
```

**Priority**: 🟢 **Recommended** - Improves discoverability

**Estimated Effort**: 2-3 hours

---

### 4.2 Web App Manifest Validator
**Status**: ⏳ Not Started

**Scope**: Validate Progressive Web App (PWA) manifest

**Implementation**:
- [ ] Look for `<link rel="manifest">` in HTML
- [ ] Fetch manifest file (usually `/manifest.json`)
- [ ] Parse and validate JSON structure
- [ ] Check for required PWA fields:
  - [ ] `name` / `short_name`
  - [ ] `icons` (with appropriate sizes)
  - [ ] `start_url`
  - [ ] `display` mode
  - [ ] `theme_color`
  - [ ] `background_color`
- [ ] Validate icon URLs are accessible
- [ ] Check icon sizes (e.g., 192x192, 512x512)
- [ ] Generate PWA readiness report

**Output**:
```markdown
## Web App Manifest

### Status: ✅ Found at `/manifest.json`

### PWA Configuration:
- **Name**: Example App
- **Short Name**: ExApp
- **Start URL**: /
- **Display**: standalone
- **Theme**: #667eea

### Icons:
- ✅ 192x192 icon present
- ✅ 512x512 icon present
- ✅ All icons accessible

### PWA Readiness: ✅ Ready for installation

### Recommendations:
- ✅ All required fields present
- Consider adding shortcuts for quick actions
```

**Priority**: 🟢 **Recommended** - Nice-to-have for PWA sites

**Estimated Effort**: 2-3 hours

---

## Phase 5: Integration & Reporting 📈

### 5.1 Unified AI Readiness Score
**Status**: ⏳ Not Started

**Scope**: Create comprehensive AI-readiness dashboard

**Implementation**:
- [ ] Aggregate all checks into unified report
- [ ] Create scoring system:
  - [ ] Essential: AI Plugin, OpenAPI, Structured Data
  - [ ] Important: Robots.txt, Sitemap, HTTP Headers
  - [ ] Recommended: Web App Manifest, Security.txt
- [ ] Calculate overall readiness percentage
- [ ] Prioritize recommendations by impact
- [ ] Generate executive summary for non-technical users

**Output**:
```markdown
# AI Readiness Report

## Overall Score: 75% - Good

### ✅ What's Working Well:
- Schema.org structured data present on 85% of pages
- XML sitemap with 1,247 URLs properly configured
- Robots.txt allows AI crawler access

### ⚠️ Areas for Improvement:
- **Critical**: No AI Plugin Manifest - site cannot be used by ChatGPT plugins
- **High**: OpenAPI spec not found - API not discoverable by AI agents
- **Medium**: Missing Web App Manifest - not installable as PWA

### 📊 Category Breakdown:
- **AI Integration**: 40% (Missing plugin manifest and OpenAPI)
- **Content Discovery**: 90% (Sitemap and robots.txt excellent)
- **Structured Data**: 85% (Good coverage, minor improvements needed)
- **Additional Features**: 60% (Some headers present, PWA not configured)

### 🎯 Top 3 Recommendations:
1. Create `/.well-known/ai-plugin.json` to enable ChatGPT integration
2. Publish OpenAPI specification for your API
3. Add Article schema to blog posts for rich search results
```

**Priority**: 🔴 **Critical** - Final deliverable

**Estimated Effort**: 3-4 hours

---

### 5.2 CLI & Worker Integration
**Status**: ⏳ Not Started

**Scope**: Integrate all checks into CLI and Worker

**Implementation**:

**CLI (`htmlens-cli`)**:
- [ ] Add new flag: `--ai-readiness` or `--check-ai`
- [ ] Run all AI readiness checks when flag is used
- [ ] Generate comprehensive markdown report
- [ ] Add flag: `--ai-report-json` for JSON output
- [ ] Add flag: `--ai-quick` for fast essential checks only

**Worker (`htmlens-worker`)**:
- [ ] Add new API endpoint: `GET /api/ai-readiness?url=<URL>`
- [ ] Add new frontend tab: "AI Readiness"
- [ ] Display AI readiness score with visual indicators
- [ ] Show category breakdown with progress bars
- [ ] List recommendations with priority badges
- [ ] Add "Quick Check" vs "Full Analysis" toggle

**Example CLI Usage**:
```bash
# Full AI readiness check
htmlens --ai-readiness https://example.com

# Quick essential checks only
htmlens --ai-quick https://example.com

# JSON output for automation
htmlens --ai-readiness --ai-report-json https://example.com > report.json
```

**Priority**: 🔴 **Critical** - User-facing interface

**Estimated Effort**: 6-8 hours

---

## Technical Architecture

### New Crate Structure

```
crates/
├── htmlens-core/
│   ├── src/
│   │   ├── types.rs              (existing)
│   │   ├── parser.rs             (existing)
│   │   ├── graph.rs              (existing)
│   │   └── ai_readiness/         ← NEW MODULE
│   │       ├── mod.rs            (public API)
│   │       ├── well_known.rs     (Phase 1.1)
│   │       ├── plugin_manifest.rs (Phase 1.2)
│   │       ├── openapi.rs        (Phase 1.3)
│   │       ├── robots.rs         (Phase 2.1)
│   │       ├── sitemap.rs        (Phase 2.2)
│   │       ├── headers.rs        (Phase 4.1)
│   │       └── manifest.rs       (Phase 4.2)
├── htmlens-cli/
│   └── src/
│       └── main.rs               (add --ai-readiness flag)
└── htmlens-worker/
    └── src/
        ├── lib.rs                (add /api/ai-readiness endpoint)
        └── frontend.html         (add AI Readiness tab)
```

### Dependencies to Add

**Cargo.toml additions**:
```toml
[dependencies]
# Existing dependencies...
openapiv3 = "2.0"        # OpenAPI spec parsing
roxmltree = "0.20"       # XML sitemap parsing
robotparser = "0.13"     # robots.txt parsing (or custom impl)
```

---

## Implementation Timeline

### Week 1: Foundation
- [ ] Phase 1.1: `.well-known/` checks (2-3h)
- [ ] Phase 1.2: AI Plugin Manifest validation (4-5h)
- [ ] Phase 1.3: OpenAPI validation (5-6h)
- **Total**: ~12-14 hours

### Week 2: Content Discovery
- [ ] Phase 2.1: Robots.txt parser (3-4h)
- [ ] Phase 2.2: XML Sitemap validator (4-5h)
- [ ] Phase 3.1: Expanded Schema.org validation (3-4h)
- **Total**: ~10-13 hours

### Week 3: Polish & Integration
- [ ] Phase 3.2: Coverage report (3-4h)
- [ ] Phase 4.1: HTTP headers (2-3h)
- [ ] Phase 4.2: Web App Manifest (2-3h)
- [ ] Phase 5.1: Unified score (3-4h)
- [ ] Phase 5.2: CLI & Worker integration (6-8h)
- **Total**: ~16-22 hours

**Total Estimated Effort**: 38-49 hours (~1 week of focused work)

---

## Success Metrics

### Functional Completeness
- [ ] All 12 sub-tasks implemented
- [ ] CLI flag `--ai-readiness` working
- [ ] Worker API endpoint `/api/ai-readiness` responding
- [ ] Frontend "AI Readiness" tab rendering

### Quality Indicators
- [ ] Handles missing files gracefully (404s)
- [ ] Validates JSON/XML formats correctly
- [ ] Provides actionable recommendations
- [ ] Non-technical manager can understand reports
- [ ] Build time remains under 10 seconds for Worker

### Testing Coverage
- [ ] Test with sites that have all features (e.g., Stripe, Shopify)
- [ ] Test with sites missing features
- [ ] Test with malformed files (invalid JSON/XML)
- [ ] Test with network errors (timeouts, 5xx responses)

---

## Future Enhancements (Post-MVP)

### Advanced Features
- [ ] Historical tracking (compare AI readiness over time)
- [ ] Competitive benchmarking (compare with similar sites)
- [ ] Automated fixes (generate AI plugin manifest template)
- [ ] Integration with Google Search Console API
- [ ] Support for other AI platforms (Claude, Gemini, etc.)
- [ ] Webhook notifications for changes
- [ ] Browser extension for quick checks

### Performance Optimizations
- [ ] Cache API responses (avoid redundant requests)
- [ ] Parallel HTTP requests (check multiple files simultaneously)
- [ ] Incremental checks (only validate changed files)
- [ ] CDN-aware checking (respect cache headers)

---

## Documentation Needs

- [ ] Update `README.md` with AI readiness features
- [ ] Update `AGENTS.md` with new module architecture
- [ ] Create `docs/AI_READINESS.md` with detailed guide
- [ ] Add examples in `examples/ai-readiness/`
- [ ] Update Worker frontend help text
- [ ] Add blog post: "How to Make Your Website AI-Ready"

---

## Notes & Considerations

### Design Decisions
1. **Feature flag approach**: Should AI readiness be a separate feature flag in `htmlens-core`?
   - **Decision**: Yes, similar to `full-expansion`. Add `ai-readiness` feature.
   - **Reasoning**: Worker may not need all checks (keep lightweight), CLI needs full suite.

2. **HTTP request strategy**: Some checks require many HTTP requests (performance concern)
   - **Decision**: Implement request pooling and parallel fetching with timeout limits.
   - **Reasoning**: User can tolerate 5-10 seconds for comprehensive check.

3. **Error handling**: How to handle sites that block automated requests?
   - **Decision**: Gracefully degrade, report "Could not check due to access restrictions".
   - **Reasoning**: Some sites use Cloudflare challenges or rate limiting.

4. **Caching**: Should we cache fetched files between runs?
   - **Decision**: No caching in MVP, add in future enhancement.
   - **Reasoning**: Simplicity first, avoid stale data issues.

### Open Questions
- [ ] Should we validate security.txt format in detail or just check existence?
- [ ] How many sitemap URLs should we spot-check (10? 100?)?
- [ ] Should we support sitemap index files with 100+ sitemaps?
- [ ] Should OpenAPI validation be strict or lenient (warnings vs errors)?

---

## Progress Tracking

**Last Updated**: October 22, 2025  
**Current Phase**: Phase 1 Complete ✅  
**Next Steps**: Begin Phase 2 - Content Discovery & Crawling

### Completed
- ✅ Read and analyzed PDF requirements
- ✅ Created comprehensive implementation plan
- ✅ Defined architecture and dependencies
- ✅ Estimated effort and timeline
- ✅ **Phase 1.1**: `.well-known/` Directory Checks - Implemented
- ✅ **Phase 1.2**: AI Plugin Manifest Validation - Implemented
- ✅ **Phase 1.3**: OpenAPI Specification Validation - Implemented
- ✅ Added `ai-readiness` feature flag to `htmlens-core`
- ✅ All Phase 1 code compiles successfully

### In Progress
- ⏳ Ready to begin Phase 2

### Blocked
- None currently
