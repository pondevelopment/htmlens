//! Cloudflare Worker for htmlens
//!
//! Pure Rust implementation that serves a web UI and extracts JSON-LD using htmlens-core

use htmlens_core::{GraphNode, normalize_origin, parser};
use serde::Serialize;
use url::Url;
use worker::*;

#[derive(Serialize)]
struct ApiResponse {
    url: String,
    title: String,
    description: String,
    graph: GraphData,
    jsonld: Vec<serde_json::Value>,
    #[serde(rename = "jsonldGraph")]
    jsonld_graph: serde_json::Value, // Combined JSON-LD as @graph
    markdown: String, // CLI-style formatted markdown with product tables
    #[serde(rename = "pageMarkdown")]
    page_markdown: String, // CF AI converted HTML page content
    meta: MetaData,
    #[serde(rename = "aiReadiness")]
    ai_readiness: AiReadinessData,
}

#[derive(Serialize)]
struct GraphData {
    nodes: Vec<GraphNode>,
    edges: Vec<String>, // Simplified for now
}

#[derive(Serialize)]
struct MetaData {
    #[serde(rename = "htmlLength")]
    html_length: usize,
    #[serde(rename = "jsonldCount")]
    jsonld_count: usize,
    #[serde(rename = "wasmStatus")]
    wasm_status: String,
}

#[derive(Serialize)]
struct AiReadinessData {
    #[serde(rename = "wellKnown")]
    well_known: WellKnownChecks,
    #[serde(rename = "aiPlugin")]
    ai_plugin: Option<AiPluginStatus>,
    mcp: Option<McpStatus>,
    openapi: Option<OpenApiStatus>,
    #[serde(rename = "robotsTxt")]
    robots_txt: Option<RobotsTxtStatus>,
    sitemap: Option<SitemapStatus>,
    #[serde(rename = "semanticHtml")]
    semantic_html: Option<SemanticHtmlStatus>,
    #[serde(rename = "securityTxt")]
    security_txt: Option<SecurityTxtInfo>,
    #[serde(rename = "llmsTxt")]
    llms_txt: Option<LlmsTxtStatus>,
    // New informational checks (no score impact).
    #[serde(rename = "linkHeaders")]
    link_headers: Option<LinkHeaderInfo>,
    #[serde(rename = "markdownNegotiation")]
    markdown_negotiation: Option<MarkdownNegotiationInfo>,
    #[serde(rename = "oauthAuthorizationServer")]
    oauth_authorization_server: Option<OauthMetadataInfo>,
    #[serde(rename = "oauthProtectedResource")]
    oauth_protected_resource: Option<OauthProtectedResourceInfo>,
    #[serde(rename = "apiCatalog")]
    api_catalog: Option<ApiCatalogInfo>,
    #[serde(rename = "mcpServerCard")]
    mcp_server_card: Option<McpServerCardInfo>,
    #[serde(rename = "webBotAuth")]
    web_bot_auth: Option<WebBotAuthInfo>,
    #[serde(rename = "agentSkills")]
    agent_skills: Option<AgentSkillsInfo>,
    commerce: CommerceChecks,
}

/// Homepage-derived signals captured from the main page fetch so we don't
/// have to re-fetch the homepage for every check that needs it.
#[allow(dead_code)]
struct HomepageSignals<'a> {
    link_header: String,
    www_authenticate: String,
    status_code: u16,
    content_type: String,
    html: &'a str,
}

#[derive(Serialize)]
struct LinkHeaderInfo {
    raw: String,
    links: Vec<ParsedLink>,
    /// Subset of links whose `rel` matches an agent-useful relation type.
    #[serde(rename = "agentUseful")]
    agent_useful: Vec<ParsedLink>,
}

#[derive(Serialize, Clone)]
struct ParsedLink {
    target: String,
    rel: String,
}

#[derive(Serialize)]
struct MarkdownNegotiationInfo {
    supported: bool,
    status: u16,
    #[serde(rename = "contentType")]
    content_type: String,
}

#[derive(Serialize)]
struct OauthMetadataInfo {
    valid: bool,
    issuer: Option<String>,
    #[serde(rename = "authorizationEndpoint")]
    authorization_endpoint: Option<String>,
    #[serde(rename = "tokenEndpoint")]
    token_endpoint: Option<String>,
    #[serde(rename = "scopesSupported")]
    scopes_supported: Vec<String>,
}

#[derive(Serialize)]
struct OauthProtectedResourceInfo {
    found: bool,
    resource: Option<String>,
    #[serde(rename = "authorizationServers")]
    authorization_servers: Vec<String>,
    #[serde(rename = "scopesSupported")]
    scopes_supported: Vec<String>,
    /// `WWW-Authenticate` header value captured from the homepage fetch.
    #[serde(rename = "wwwAuthenticate")]
    www_authenticate: Option<String>,
}

#[derive(Serialize)]
struct ApiCatalogInfo {
    valid: bool,
    #[serde(rename = "entryCount")]
    entry_count: usize,
    entries: Vec<ApiCatalogEntry>,
    issues: Vec<String>,
}

#[derive(Serialize)]
struct ApiCatalogEntry {
    anchor: String,
    relations: Vec<String>,
}

#[derive(Serialize)]
struct McpServerCardInfo {
    found: bool,
    path: String,
    name: Option<String>,
    version: Option<String>,
    transport: Option<String>,
    endpoint: Option<String>,
}

#[derive(Serialize)]
struct WebBotAuthInfo {
    found: bool,
    status: u16,
    #[serde(rename = "contentType")]
    content_type: String,
}

#[derive(Serialize)]
struct AgentSkillsInfo {
    found: bool,
    path: String,
    #[serde(rename = "skillCount")]
    skill_count: usize,
    skills: Vec<AgentSkillEntry>,
}

#[derive(Serialize)]
struct AgentSkillEntry {
    name: String,
    #[serde(rename = "type")]
    skill_type: Option<String>,
    description: Option<String>,
    url: Option<String>,
}

#[derive(Serialize, Default)]
struct CommerceChecks {
    ucp: Option<UcpInfo>,
    acp: Option<AcpInfo>,
    mpp: Option<MppInfo>,
    x402: Option<X402Info>,
}

#[derive(Serialize)]
struct UcpInfo {
    found: bool,
    version: Option<String>,
    services: Vec<String>,
}

#[derive(Serialize)]
struct AcpInfo {
    found: bool,
    version: Option<String>,
    #[serde(rename = "apiBaseUrl")]
    api_base_url: Option<String>,
}

#[derive(Serialize)]
struct MppInfo {
    found: bool,
    #[serde(rename = "operationCount")]
    operation_count: usize,
    #[serde(rename = "payableOperations")]
    payable_operations: usize,
}

#[derive(Serialize)]
struct X402Info {
    detected: bool,
    probes: Vec<X402Probe>,
}

#[derive(Serialize)]
struct X402Probe {
    path: String,
    status: u16,
}

#[derive(Serialize)]
struct WellKnownChecks {
    #[serde(rename = "aiPluginJson")]
    ai_plugin_json: FileStatus,
    #[serde(rename = "mcpJson")]
    mcp_json: FileStatus,
    #[serde(rename = "openidConfiguration")]
    openid_configuration: FileStatus,
    #[serde(rename = "securityTxt")]
    security_txt: FileStatus,
    #[serde(rename = "appleAppSiteAssociation")]
    apple_app_site_association: FileStatus,
    assetlinks: FileStatus,
    #[serde(rename = "llmsTxt")]
    llms_txt: FileStatus,
    // New entries (informational only, no scoring impact).
    #[serde(rename = "oauthAuthorizationServer")]
    oauth_authorization_server: FileStatus,
    #[serde(rename = "oauthProtectedResource")]
    oauth_protected_resource: FileStatus,
    #[serde(rename = "httpMessageSignaturesDirectory")]
    http_message_signatures_directory: FileStatus,
    #[serde(rename = "apiCatalog")]
    api_catalog: FileStatus,
    #[serde(rename = "mcpServerCard")]
    mcp_server_card: FileStatus,
    #[serde(rename = "mcpServerCards")]
    mcp_server_cards: FileStatus,
    #[serde(rename = "agentSkillsIndex")]
    agent_skills_index: FileStatus,
    #[serde(rename = "agentSkillsLegacy")]
    agent_skills_legacy: FileStatus,
    ucp: FileStatus,
    #[serde(rename = "acpJson")]
    acp_json: FileStatus,
}

#[derive(Serialize)]
struct FileStatus {
    found: bool,
    status: u16,         // HTTP status code
    valid: Option<bool>, // JSON validity if applicable
}

#[derive(Serialize)]
struct AiPluginStatus {
    valid: bool,
    name: String,
    description: String,
    #[serde(rename = "hasAuth")]
    has_auth: bool,
    #[serde(rename = "apiUrl")]
    api_url: Option<String>,
    issues: Vec<String>,
}

#[derive(Serialize)]
struct McpStatus {
    valid: bool,
    name: String,
    version: String,
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    #[serde(rename = "transportType")]
    transport_type: String,
    endpoint: String,
    #[serde(rename = "toolCount")]
    tool_count: usize,
    #[serde(rename = "resourceCount")]
    resource_count: usize,
    #[serde(rename = "promptCount")]
    prompt_count: usize,
    #[serde(rename = "hasToolsCapability")]
    has_tools_capability: bool,
    #[serde(rename = "hasResourcesCapability")]
    has_resources_capability: bool,
    #[serde(rename = "hasPromptsCapability")]
    has_prompts_capability: bool,
    #[serde(rename = "hasEventsCapability")]
    has_events_capability: bool,
    #[serde(rename = "healthEndpoint")]
    health_endpoint: Option<String>,
    issues: Vec<String>,
}

#[derive(Serialize)]
struct OpenApiStatus {
    valid: bool,
    version: String,
    #[serde(rename = "operationCount")]
    operation_count: usize,
    #[serde(rename = "hasAuth")]
    has_auth: bool,
    issues: Vec<String>,
}

#[derive(Serialize)]
struct RobotsTxtStatus {
    found: bool,
    #[serde(rename = "sitemapCount")]
    sitemap_count: usize,
    sitemaps: Vec<String>,
    #[serde(rename = "aiCrawlers")]
    ai_crawlers: Vec<AiCrawlerInfo>,
    #[serde(rename = "blocksAllBots")]
    blocks_all_bots: bool,
    issues: Vec<String>,
    /// Content Signals (contentsignals.org / IETF draft) parsed from
    /// `Content-Signal:` directives in robots.txt. Informational.
    #[serde(rename = "contentSignals")]
    content_signals: ContentSignals,
}

#[derive(Serialize, Default)]
struct ContentSignals {
    found: bool,
    #[serde(rename = "aiTrain")]
    ai_train: Option<String>,
    search: Option<String>,
    #[serde(rename = "aiInput")]
    ai_input: Option<String>,
    raw: Vec<String>,
}

#[derive(Serialize)]
struct AiCrawlerInfo {
    name: String,
    access: String, // "allowed", "partial", "blocked", "default"
    rules: Option<String>,
}

#[derive(Serialize)]
struct SitemapStatus {
    found: bool,
    #[serde(rename = "sitemapType")]
    sitemap_type: String, // "standard", "index", "unknown"
    #[serde(rename = "urlCount")]
    url_count: usize,
    #[serde(rename = "lastModified")]
    last_modified: Option<String>,
    statistics: SitemapStats,
    #[serde(rename = "nestedSitemaps")]
    nested_sitemaps: Vec<String>,
    #[serde(rename = "sampleUrls")]
    sample_urls: Vec<SitemapUrlEntry>,
    issues: Vec<String>,
    recommendations: Vec<String>,
}

#[derive(Serialize)]
struct SitemapStats {
    #[serde(rename = "totalUrls")]
    total_urls: usize,
    #[serde(rename = "urlsWithLastmod")]
    urls_with_lastmod: usize,
    #[serde(rename = "urlsWithPriority")]
    urls_with_priority: usize,
    #[serde(rename = "avgPriority")]
    avg_priority: f32,
    #[serde(rename = "contentTypes")]
    content_types: std::collections::HashMap<String, usize>,
}

#[derive(Serialize)]
struct SitemapUrlEntry {
    loc: String,
    lastmod: Option<String>,
    changefreq: Option<String>,
    priority: Option<f32>,
}

#[derive(Serialize)]
struct SemanticHtmlStatus {
    landmarks: LandmarksInfo,
    headings: HeadingsInfo,
    sections: SectionsInfo,
    figures: FiguresInfo,
    links: LinksInfo,
    #[serde(rename = "elementUsage")]
    element_usage: ElementUsageInfo,
    aria: AriaInfo,
    forms: FormsInfo,
    images: ImagesInfo,
    issues: Vec<String>,
    recommendations: Vec<String>,
}

#[derive(Serialize)]
struct LandmarksInfo {
    #[serde(rename = "hasMain")]
    has_main: bool,
    #[serde(rename = "mainCount")]
    main_count: usize,
    #[serde(rename = "hasSingleMain")]
    has_single_main: bool,
    #[serde(rename = "invalidMainNestingCount")]
    invalid_main_nesting_count: usize,
    #[serde(rename = "hasNavigation")]
    has_navigation: bool,
    #[serde(rename = "hasHeader")]
    has_header: bool,
    #[serde(rename = "hasFooter")]
    has_footer: bool,
    #[serde(rename = "articleCount")]
    article_count: usize,
    #[serde(rename = "sectionCount")]
    section_count: usize,
    #[serde(rename = "asideCount")]
    aside_count: usize,
}

#[derive(Serialize)]
struct HeadingsInfo {
    #[serde(rename = "hasSingleH1")]
    has_single_h1: bool,
    #[serde(rename = "properHierarchy")]
    proper_hierarchy: bool,
    distribution: Vec<usize>,
    #[serde(rename = "hierarchyIssues")]
    hierarchy_issues: Vec<String>,
}

#[derive(Serialize)]
struct SectionsInfo {
    #[serde(rename = "sectionsWithoutHeadings")]
    sections_without_headings: usize,
    #[serde(rename = "articlesWithoutHeadings")]
    articles_without_headings: usize,
}

#[derive(Serialize)]
struct FiguresInfo {
    #[serde(rename = "totalFigures")]
    total_figures: usize,
    #[serde(rename = "figuresWithCaptions")]
    figures_with_captions: usize,
    #[serde(rename = "figuresWithoutCaptions")]
    figures_without_captions: usize,
}

#[derive(Serialize)]
struct LinksInfo {
    #[serde(rename = "totalLinks")]
    total_links: usize,
    #[serde(rename = "linksWithoutNames")]
    links_without_names: usize,
    #[serde(rename = "genericTextLinks")]
    generic_text_links: usize,
}

#[derive(Serialize)]
struct ElementUsageInfo {
    #[serde(rename = "divCount")]
    div_count: usize,
    #[serde(rename = "semanticContainerCount")]
    semantic_container_count: usize,
}

#[derive(Serialize)]
struct AriaInfo {
    #[serde(rename = "labeledElements")]
    labeled_elements: usize,
    #[serde(rename = "describedElements")]
    described_elements: usize,
    #[serde(rename = "liveRegions")]
    live_regions: usize,
    #[serde(rename = "interactiveRoles")]
    interactive_roles: usize,
    #[serde(rename = "misuseWarnings")]
    misuse_warnings: Vec<String>,
}

#[derive(Serialize)]
struct FormsInfo {
    #[serde(rename = "formCount")]
    form_count: usize,
    #[serde(rename = "totalInputs")]
    total_inputs: usize,
    #[serde(rename = "labeledInputs")]
    labeled_inputs: usize,
    #[serde(rename = "labelPercentage")]
    label_percentage: u32,
    #[serde(rename = "formsWithFieldsets")]
    forms_with_fieldsets: usize,
    #[serde(rename = "requiredFieldsMarked")]
    required_fields_marked: bool,
}

#[derive(Serialize)]
struct ImagesInfo {
    #[serde(rename = "totalImages")]
    total_images: usize,
    #[serde(rename = "imagesWithAlt")]
    images_with_alt: usize,
    #[serde(rename = "decorativeImages")]
    decorative_images: usize,
    #[serde(rename = "imagesMissingAlt")]
    images_missing_alt: usize,
    #[serde(rename = "altPercentage")]
    alt_percentage: u32,
}

#[derive(Serialize)]
struct SecurityTxtInfo {
    contacts: Vec<String>,
    expires: Option<String>,
    encryption: Vec<String>,
    acknowledgments: Vec<String>,
    #[serde(rename = "preferredLanguages")]
    preferred_languages: Option<String>,
    policy: Vec<String>,
    hiring: Vec<String>,
    canonical: Vec<String>,
    #[serde(rename = "isExpired")]
    is_expired: bool,
}

#[derive(Serialize)]
struct LlmsTxtStatus {
    found: bool,
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "sectionCount")]
    section_count: usize,
    #[serde(rename = "linkCount")]
    link_count: usize,
    #[serde(rename = "hasOptionalSection")]
    has_optional_section: bool,
    sections: Vec<String>,
}

// Frontend HTML will be included as a separate file
const FRONTEND_HTML: &str = include_str!("frontend.html");

fn extract_title(html: &str) -> String {
    // Simple regex-based title extraction
    if let Some(start) = html.find("<title")
        && let Some(content_start) = html[start..].find('>')
    {
        let content_start = start + content_start + 1;
        if let Some(end) = html[content_start..].find("</title>") {
            return html[content_start..content_start + end].trim().to_string();
        }
    }
    "Unknown Title".to_string()
}

fn extract_description(html: &str) -> String {
    // Simple meta description extraction
    if let Some(start) = html
        .find(r#"name="description""#)
        .or_else(|| html.find(r#"name='description'"#))
        && let Some(content) = html[start..].find("content=")
    {
        let content_start = start + content + 8;
        let quote = html.chars().nth(content_start).unwrap_or('"');
        if let Some(end) = html[content_start + 1..].find(quote) {
            return html[content_start + 1..content_start + 1 + end].to_string();
        }
    }
    String::new()
}

/// Browser-like User-Agent used for all AI readiness sub-fetches.
/// Many sites (Shopify, Cloudflare-fronted, Akamai) return 403 to the default
/// Cloudflare Workers User-Agent, so we present as a normal crawler.
const FETCH_UA: &str =
    "Mozilla/5.0 (compatible; HTMLens/0.4.0; +https://github.com/pondevelopment/htmlens)";

/// Default cache TTL in seconds for `.well-known`, robots.txt, llms.txt, OAuth
/// metadata, etc. Short-lived since these files do change occasionally.
const CACHE_TTL_DEFAULT: u32 = 300; // 5 minutes
/// Longer TTL for sitemaps (XML files rarely change between scans).
#[allow(dead_code)]
const CACHE_TTL_SITEMAP: u32 = 900; // 15 minutes
/// Short TTL for x402 probes (status code may flip when site adds payments).
const CACHE_TTL_X402: u32 = 60; // 1 minute

// Per-request flag: when true, sub-fetches bypass the edge cache. Set by the
// main handler from the `?nocache=1` query parameter. Workers are single-
// threaded so a thread-local Cell is safe.
thread_local! {
    static BYPASS_CACHE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn set_bypass_cache(bypass: bool) {
    BYPASS_CACHE.with(|b| b.set(bypass));
}

fn is_bypass_cache() -> bool {
    BYPASS_CACHE.with(|b| b.get())
}

/// Fetch a URL with a browser-like User-Agent and (optionally) an Accept
/// header, going through the Cloudflare edge Cache API.
///
/// Behaviour:
/// - On cache HIT: returns the cached response without contacting the origin.
/// - On cache MISS or bypass: fetches the origin, and on HTTP 200 stores the
///   response with `Cache-Control: public, max-age=<ttl>`.
/// - Non-200 responses are NEVER cached (avoids locking in transient 403/5xx).
async fn fetch_with_ua_opts(url: &str, accept: Option<&str>, ttl: u32) -> Option<Response> {
    let bypass = is_bypass_cache();
    let cache = Cache::default();

    // Compose a cache key URL. Distinguish by Accept by appending a synthetic
    // query parameter — Cloudflare's Cache API keys on the request URL, and
    // separating variants by URL is the simplest correct way to do this.
    let cache_key_url = match accept {
        Some(_) => format!(
            "{}{}_hcache_accept={}",
            url,
            if url.contains('?') { "&" } else { "?" },
            urlencoding::encode(accept.unwrap_or(""))
        ),
        None => url.to_string(),
    };

    if !bypass {
        if let Ok(Some(hit)) = cache.get(cache_key_url.as_str(), false).await {
            console_log!("[Cache] HIT {}", url);
            return Some(hit);
        }
    } else {
        let _ = cache.delete(cache_key_url.as_str(), false).await;
    }

    let mut req = Request::new(url, Method::Get).ok()?;
    {
        let headers = req.headers_mut().ok()?;
        headers.set("User-Agent", FETCH_UA).ok()?;
        if let Some(a) = accept {
            headers.set("Accept", a).ok()?;
        }
    }
    let mut response = Fetch::Request(req).send().await.ok()?;

    let status = response.status_code();
    if status == 200 {
        // Clone, attach Cache-Control + Vary, and store.
        if let Ok(mut cacheable) = response.cloned() {
            {
                let headers = cacheable.headers_mut();
                let _ = headers.set("Cache-Control", &format!("public, max-age={}", ttl));
                if accept.is_some() {
                    let _ = headers.set("Vary", "Accept");
                }
            }
            let _ = cache.put(cache_key_url.as_str(), cacheable).await;
        }
    }
    Some(response)
}

/// Convenience wrapper: default TTL, no custom Accept header.
async fn fetch_with_ua(url: &str) -> Option<Response> {
    fetch_with_ua_opts(url, None, CACHE_TTL_DEFAULT).await
}

async fn check_ai_readiness(base_url: &str, signals: HomepageSignals<'_>) -> AiReadinessData {
    let origin = normalize_origin(base_url);

    // Parse Link header from the homepage fetch (no extra HTTP).
    let link_headers = if signals.link_header.is_empty() {
        None
    } else {
        Some(parse_link_header(&signals.link_header))
    };

    // Markdown negotiation: re-fetch homepage with Accept: text/markdown.
    let markdown_negotiation = check_markdown_negotiation(base_url).await;

    // Check .well-known files
    let well_known = check_well_known_files(&origin).await;

    // Check AI Plugin if found
    let ai_plugin =
        if well_known.ai_plugin_json.found && well_known.ai_plugin_json.valid == Some(true) {
            check_ai_plugin(&origin).await
        } else {
            None
        };

    // Check MCP if found
    let mcp = if well_known.mcp_json.found && well_known.mcp_json.valid == Some(true) {
        check_mcp(&origin).await
    } else {
        None
    };

    // Check OpenAPI if AI plugin references it
    let openapi = if let Some(ref plugin) = ai_plugin {
        if let Some(ref api_url) = plugin.api_url {
            check_openapi(api_url).await
        } else {
            None
        }
    } else {
        None
    };

    // Check robots.txt
    let robots_txt = check_robots_txt(&origin).await;

    // Check sitemap (use URLs from robots.txt if available)
    let sitemap_urls = if let Some(ref robots) = robots_txt {
        robots.sitemaps.clone()
    } else {
        Vec::new()
    };
    let sitemap = check_sitemap(&origin, sitemap_urls).await;

    // Analyze the same homepage HTML used for the rest of the report.
    let semantic_html =
        is_semantic_html_response(signals.status_code, &signals.content_type, signals.html)
            .then(|| build_semantic_html_status(signals.html));

    // Parse security.txt content if found
    let security_txt = if well_known.security_txt.found {
        parse_security_txt(&origin).await
    } else {
        None
    };

    // Check llms.txt
    let llms_txt = check_llms_txt(&origin).await;

    // ---- New informational deep validators (no scoring impact) ----
    let oauth_authorization_server = if well_known.oauth_authorization_server.found {
        check_oauth_authorization_server(&origin).await
    } else {
        None
    };
    let oauth_protected_resource =
        if well_known.oauth_protected_resource.found || !signals.www_authenticate.is_empty() {
            check_oauth_protected_resource(&origin, &signals.www_authenticate).await
        } else {
            None
        };
    let api_catalog = if well_known.api_catalog.found {
        check_api_catalog(&origin).await
    } else {
        None
    };
    let mcp_server_card = if well_known.mcp_server_card.found || well_known.mcp_server_cards.found {
        check_mcp_server_card(&origin).await
    } else {
        None
    };
    let agent_skills =
        if well_known.agent_skills_index.found || well_known.agent_skills_legacy.found {
            check_agent_skills(&origin).await
        } else {
            None
        };
    let web_bot_auth = check_web_bot_auth(&origin).await;

    let commerce = CommerceChecks {
        ucp: if well_known.ucp.found {
            check_ucp(&origin).await
        } else {
            None
        },
        acp: if well_known.acp_json.found {
            check_acp(&origin).await
        } else {
            None
        },
        mpp: None, // MPP discovery not yet defined; placeholder slot.
        x402: check_x402(&origin).await,
    };

    AiReadinessData {
        well_known,
        ai_plugin,
        mcp,
        openapi,
        robots_txt,
        sitemap,
        semantic_html,
        security_txt,
        llms_txt,
        link_headers,
        markdown_negotiation,
        oauth_authorization_server,
        oauth_protected_resource,
        api_catalog,
        mcp_server_card,
        web_bot_auth,
        agent_skills,
        commerce,
    }
}

fn is_semantic_html_response(status_code: u16, content_type: &str, body: &str) -> bool {
    if status_code != 200 || body.trim().is_empty() {
        return false;
    }

    let media_type = content_type.split(';').next().unwrap_or_default().trim();
    if !media_type.is_empty() {
        return media_type.eq_ignore_ascii_case("text/html")
            || media_type.eq_ignore_ascii_case("application/xhtml+xml");
    }

    let prefix = body
        .trim_start()
        .chars()
        .take(1024)
        .collect::<String>()
        .to_ascii_lowercase();
    prefix.contains("<!doctype html") || prefix.contains("<html")
}

async fn check_well_known_files(base_url: &str) -> WellKnownChecks {
    console_log!(
        "[AI Readiness] Checking .well-known files for: {}",
        base_url
    );

    let files = vec![
        ("ai-plugin.json", "aiPluginJson"),
        ("mcp.json", "mcpJson"),
        ("openid-configuration", "openidConfiguration"),
        ("security.txt", "securityTxt"),
        ("apple-app-site-association", "appleAppSiteAssociation"),
        ("assetlinks.json", "assetlinks"),
    ];

    let mut results = WellKnownChecks {
        ai_plugin_json: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        mcp_json: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        openid_configuration: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        security_txt: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        apple_app_site_association: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        assetlinks: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        llms_txt: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        oauth_authorization_server: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        oauth_protected_resource: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        http_message_signatures_directory: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        api_catalog: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        mcp_server_card: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        mcp_server_cards: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        agent_skills_index: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        agent_skills_legacy: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        ucp: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
        acp_json: FileStatus {
            found: false,
            status: 0,
            valid: None,
        },
    };

    for (filename, _) in files {
        let url = format!("{}/.well-known/{}", base_url, filename);
        console_log!("[.well-known] Checking: {}", url);

        // Create request with redirect set to "manual" to prevent following redirects
        let mut init = RequestInit::new();
        init.with_redirect(RequestRedirect::Manual);

        let request_result = Request::new_with_init(&url, &init).and_then(|mut r| {
            r.headers_mut()?.set("User-Agent", FETCH_UA)?;
            Ok(r)
        });

        if let Ok(request) = request_result
            && let Ok(mut response) = Fetch::Request(request).send().await
        {
            let status = response.status_code();

            // First check: HTTP status must be 200 (success)
            // A 404/403/etc should never be considered "found" even if valid JSON
            if status != 200 {
                console_log!("[.well-known] {} - Not found (HTTP {})", filename, status);
                let file_status = FileStatus {
                    found: false,
                    status,
                    valid: None,
                };
                match filename {
                    "ai-plugin.json" => results.ai_plugin_json = file_status,
                    "mcp.json" => results.mcp_json = file_status,
                    "openid-configuration" => results.openid_configuration = file_status,
                    "security.txt" => results.security_txt = file_status,
                    "apple-app-site-association" => {
                        results.apple_app_site_association = file_status
                    }
                    "assetlinks.json" => results.assetlinks = file_status,
                    _ => {}
                }
                continue;
            }

            // For JSON files, validate the content
            // (some servers return 200 with HTML error pages)
            let (found, valid) = if filename.ends_with(".json") 
                    || filename == "openid-configuration"  // OpenID config is JSON even without .json extension
                    || filename == "apple-app-site-association"
            // Apple Universal Links config is JSON
            {
                if let Ok(text) = response.text().await {
                    // Check if it's actually valid JSON
                    if serde_json::from_str::<serde_json::Value>(&text).is_ok() {
                        console_log!("[.well-known] {} - Valid JSON found", filename);
                        (true, Some(true))
                    } else {
                        // Status was 200 but content is not valid JSON (likely error page)
                        console_log!(
                            "[.well-known] {} - Invalid JSON (likely redirected to error page)",
                            filename
                        );
                        (false, Some(false))
                    }
                } else {
                    console_log!("[.well-known] {} - Failed to read response body", filename);
                    (false, None)
                }
            } else {
                // For non-JSON files (like security.txt), status 200 means found
                (true, None)
            };

            console_log!(
                "[AI Readiness] {} - Status: {}, Found: {}, Valid: {:?}",
                filename,
                status,
                found,
                valid
            );

            let file_status = FileStatus {
                found,
                status,
                valid,
            };

            match filename {
                "ai-plugin.json" => results.ai_plugin_json = file_status,
                "mcp.json" => results.mcp_json = file_status,
                "openid-configuration" => results.openid_configuration = file_status,
                "security.txt" => results.security_txt = file_status,
                "apple-app-site-association" => results.apple_app_site_association = file_status,
                "assetlinks.json" => results.assetlinks = file_status,
                _ => {}
            }
        }
    }

    // Check /llms.txt (at root, not .well-known)
    {
        let url = format!("{}/llms.txt", base_url);
        console_log!("[AI Readiness] Checking: {}", url);

        let mut init = RequestInit::new();
        init.with_redirect(RequestRedirect::Manual);

        if let Ok(request) = Request::new_with_init(&url, &init)
            && let Ok(mut response) = Fetch::Request(request).send().await
        {
            let status = response.status_code();
            if status == 200 {
                if let Ok(text) = response.text().await {
                    // Validate: llms.txt should be markdown starting with # heading
                    let is_valid = text.trim().starts_with('#');
                    console_log!(
                        "[AI Readiness] llms.txt - Found (valid markdown: {})",
                        is_valid
                    );
                    results.llms_txt = FileStatus {
                        found: true,
                        status,
                        valid: Some(is_valid),
                    };
                }
            } else {
                console_log!("[AI Readiness] llms.txt - Not found (HTTP {})", status);
                results.llms_txt = FileStatus {
                    found: false,
                    status,
                    valid: None,
                };
            }
        }
    }

    // ------------------------------------------------------------------
    // Additional informational .well-known probes (no scoring impact).
    // These all flow through the cached fetch helper.
    // ------------------------------------------------------------------
    let new_entries: Vec<(&str, &str, Option<&str>, bool)> = vec![
        // (path, slot, accept, json_validated)
        (
            "/.well-known/oauth-authorization-server",
            "oauth-as",
            None,
            true,
        ),
        (
            "/.well-known/oauth-protected-resource",
            "oauth-pr",
            None,
            true,
        ),
        (
            "/.well-known/http-message-signatures-directory",
            "hmsd",
            None,
            true,
        ),
        (
            "/.well-known/api-catalog",
            "api-catalog",
            Some("application/linkset+json, application/json"),
            true,
        ),
        ("/.well-known/mcp/server-card.json", "mcp-card", None, true),
        (
            "/.well-known/mcp/server-cards.json",
            "mcp-cards",
            None,
            true,
        ),
        (
            "/.well-known/agent-skills/index.json",
            "skills-new",
            None,
            true,
        ),
        (
            "/.well-known/skills/index.json",
            "skills-legacy",
            None,
            true,
        ),
        ("/.well-known/ucp", "ucp", None, false),
        ("/.well-known/acp.json", "acp", None, true),
    ];
    for (path, slot, accept, json_validated) in new_entries {
        let url = format!("{}{}", base_url, path);
        console_log!("[.well-known] Checking (extra): {}", url);
        let mut status: u16 = 0;
        let mut found = false;
        let mut valid: Option<bool> = None;
        if let Some(mut response) = fetch_with_ua_opts(&url, accept, CACHE_TTL_DEFAULT).await {
            status = response.status_code();
            if status == 200 {
                if json_validated {
                    if let Ok(text) = response.text().await {
                        if serde_json::from_str::<serde_json::Value>(&text).is_ok() {
                            found = true;
                            valid = Some(true);
                        } else {
                            found = false;
                            valid = Some(false);
                        }
                    }
                } else {
                    found = true;
                }
            }
        }
        let fs = FileStatus {
            found,
            status,
            valid,
        };
        match slot {
            "oauth-as" => results.oauth_authorization_server = fs,
            "oauth-pr" => results.oauth_protected_resource = fs,
            "hmsd" => results.http_message_signatures_directory = fs,
            "api-catalog" => results.api_catalog = fs,
            "mcp-card" => results.mcp_server_card = fs,
            "mcp-cards" => results.mcp_server_cards = fs,
            "skills-new" => results.agent_skills_index = fs,
            "skills-legacy" => results.agent_skills_legacy = fs,
            "ucp" => results.ucp = fs,
            "acp" => results.acp_json = fs,
            _ => {}
        }
    }

    results
}

async fn check_ai_plugin(base_url: &str) -> Option<AiPluginStatus> {
    let url = format!("{}/.well-known/ai-plugin.json", base_url);
    if Url::parse(&url).is_ok()
        && let Some(mut response) = fetch_with_ua(&url).await
        && let Ok(text) = response.text().await
        && let Ok(plugin) = serde_json::from_str::<serde_json::Value>(&text)
    {
        let mut issues = Vec::new();

        let name = plugin
            .get("name_for_human")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let description = plugin
            .get("description_for_human")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let has_auth = plugin
            .get("auth")
            .and_then(|a| a.get("type"))
            .and_then(|t| t.as_str())
            .map(|t| t != "none")
            .unwrap_or(false);

        let api_url = plugin
            .get("api")
            .and_then(|a| a.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        if api_url.is_none() {
            issues.push("Missing API URL".to_string());
        }

        return Some(AiPluginStatus {
            valid: issues.is_empty(),
            name,
            description,
            has_auth,
            api_url,
            issues,
        });
    }
    None
}

async fn check_mcp(base_url: &str) -> Option<McpStatus> {
    let url = format!("{}/.well-known/mcp.json", base_url);
    if Url::parse(&url).is_ok()
        && let Some(mut response) = fetch_with_ua(&url).await
        && let Ok(text) = response.text().await
        && let Ok(mcp) = serde_json::from_str::<serde_json::Value>(&text)
    {
        let mut issues = Vec::new();

        let name = mcp
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let version = mcp
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let protocol_version = mcp
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let transport = mcp.get("transport");
        let transport_type = transport
            .and_then(|t| t.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown")
            .to_string();

        let endpoint = transport
            .and_then(|t| t.get("endpoint"))
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string();

        if endpoint.is_empty() {
            issues.push("Missing transport endpoint".to_string());
        }

        let capabilities = mcp.get("capabilities");
        let has_tools_capability = capabilities.and_then(|c| c.get("tools")).is_some();
        let has_resources_capability = capabilities.and_then(|c| c.get("resources")).is_some();
        let has_prompts_capability = capabilities.and_then(|c| c.get("prompts")).is_some();
        let has_events_capability = capabilities.and_then(|c| c.get("events")).is_some();

        let tools = mcp
            .get("tools")
            .and_then(|t| t.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        let resources = mcp
            .get("resources")
            .and_then(|r| r.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        let prompts = mcp
            .get("prompts")
            .and_then(|p| p.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        let health_endpoint = mcp
            .get("health")
            .and_then(|h| h.get("endpoint"))
            .and_then(|e| e.as_str())
            .map(|s| s.to_string());

        return Some(McpStatus {
            valid: issues.is_empty(),
            name,
            version,
            protocol_version,
            transport_type,
            endpoint,
            tool_count: tools,
            resource_count: resources,
            prompt_count: prompts,
            has_tools_capability,
            has_resources_capability,
            has_prompts_capability,
            has_events_capability,
            health_endpoint,
            issues,
        });
    }
    None
}

async fn check_openapi(api_url: &str) -> Option<OpenApiStatus> {
    if Url::parse(api_url).is_ok()
        && let Some(mut response) = fetch_with_ua(api_url).await
        && let Ok(text) = response.text().await
        && let Ok(spec) = serde_json::from_str::<serde_json::Value>(&text)
    {
        let mut issues = Vec::new();

        let version = spec
            .get("openapi")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let paths = spec
            .get("paths")
            .and_then(|p| p.as_object())
            .map(|p| p.len())
            .unwrap_or(0);

        let has_auth = spec
            .get("components")
            .and_then(|c| c.get("securitySchemes"))
            .is_some();

        if paths == 0 {
            issues.push("No API paths defined".to_string());
        }

        return Some(OpenApiStatus {
            valid: issues.is_empty(),
            version,
            operation_count: paths,
            has_auth,
            issues,
        });
    }
    None
}

async fn check_robots_txt(base_url: &str) -> Option<RobotsTxtStatus> {
    // robots.txt must be located at the origin
    let origin = normalize_origin(base_url);
    let robots_url = format!("{}/robots.txt", origin);
    console_log!("[AI Readiness] Checking robots.txt: {}", robots_url);
    Url::parse(&robots_url).ok()?;

    if let Some(mut response) = fetch_with_ua(&robots_url).await {
        let status = response.status_code();
        if status == 200
            && let Ok(text) = response.text().await
        {
            console_log!("[AI Readiness] robots.txt - Found ({} bytes)", text.len());
            // Parse robots.txt content
            let lines: Vec<&str> = text.lines().collect();
            let mut sitemaps = Vec::new();
            let mut ai_crawlers = Vec::new();
            let mut blocks_all_bots = false;
            let mut issues = Vec::new();

            // Extract sitemaps
            for line in &lines {
                let line = line.trim();
                if line.to_lowercase().starts_with("sitemap:") {
                    // Split only on first colon to preserve URL scheme (https:)
                    if let Some(colon_pos) = line.find(':') {
                        let sitemap_url = &line[colon_pos + 1..];
                        sitemaps.push(sitemap_url.trim().to_string());
                    }
                }
            }

            // Check for wildcard block
            let mut in_wildcard = false;
            for line in &lines {
                let line = line.trim().to_lowercase();
                if line.starts_with("user-agent:") {
                    in_wildcard = line.contains("*");
                } else if in_wildcard && line == "disallow: /" {
                    blocks_all_bots = true;
                    issues.push("Warning: All bots blocked with 'Disallow: /'".to_string());
                }
            }

            // Check AI crawlers
            let ai_crawler_names = vec![
                "GPTBot",
                "ChatGPT-User",
                "ClaudeBot",
                "Claude-Web",
                "Google-Extended",
                "Bingbot",
                "Applebot",
                "PerplexityBot",
            ];

            for crawler in ai_crawler_names {
                let access = check_crawler_access(&text, crawler);
                let rules = get_crawler_rules(&text, crawler);

                ai_crawlers.push(AiCrawlerInfo {
                    name: crawler.to_string(),
                    access: access.clone(),
                    rules,
                });

                if access == "blocked" {
                    issues.push(format!("{} is blocked from accessing the site", crawler));
                }
            }

            if sitemaps.is_empty() {
                issues.push("No sitemap URLs found in robots.txt".to_string());
            }

            // Parse Content-Signal: directives (contentsignals.org / IETF draft).
            // Lines look like `Content-Signal: ai-train=no, search=yes`.
            let mut content_signals = ContentSignals::default();
            for line in &lines {
                let raw_line = line.trim();
                let lower = raw_line.to_lowercase();
                if let Some(rest) = lower.strip_prefix("content-signal:") {
                    content_signals.found = true;
                    content_signals.raw.push(raw_line.to_string());
                    for token in rest.split(',') {
                        let token = token.trim();
                        if let Some((k, v)) = token.split_once('=') {
                            let v = v.trim().to_string();
                            match k.trim() {
                                "ai-train" => content_signals.ai_train = Some(v),
                                "search" => content_signals.search = Some(v),
                                "ai-input" => content_signals.ai_input = Some(v),
                                _ => {}
                            }
                        }
                    }
                }
            }

            return Some(RobotsTxtStatus {
                found: true,
                sitemap_count: sitemaps.len(),
                sitemaps,
                ai_crawlers,
                blocks_all_bots,
                issues,
                content_signals,
            });
        } else {
            console_log!("[AI Readiness] robots.txt - Not found (HTTP {})", status);
        }
    } else {
        console_log!("[AI Readiness] robots.txt - Fetch failed");
    }

    Some(RobotsTxtStatus {
        found: false,
        sitemap_count: 0,
        sitemaps: Vec::new(),
        ai_crawlers: Vec::new(),
        blocks_all_bots: false,
        issues: vec!["robots.txt not found".to_string()],
        content_signals: ContentSignals::default(),
    })
}

async fn check_sitemap(base_url: &str, sitemap_urls: Vec<String>) -> Option<SitemapStatus> {
    use htmlens_core::ai_readiness::sitemap;

    // Extract root domain from URL
    let root_origin = normalize_origin(base_url);
    Url::parse(&root_origin).ok()?;

    console_log!(
        "[AI Readiness] Checking sitemap ({} URL(s) from robots.txt)",
        sitemap_urls.len()
    );

    // Try sitemap URLs from robots.txt first
    for sitemap_url in &sitemap_urls {
        console_log!("[Sitemap] Trying: {}", sitemap_url);
        if Url::parse(sitemap_url).is_ok()
            && let Some(mut response) = fetch_with_ua(sitemap_url).await
            && response.status_code() == 200
            && let Ok(xml_content) = response.text().await
            && let Ok(mut analysis) = sitemap::parse_sitemap(&xml_content, &root_origin)
        {
            // If this is a sitemap index with nested sitemaps, fetch them to get accurate counts
            if !analysis.nested_sitemaps.is_empty() {
                let mut total_urls = 0;
                let mut urls_with_lastmod = 0;
                let mut urls_with_priority = 0;
                let mut priority_sum = 0.0;
                let mut all_sample_urls = Vec::new();

                for nested_url in &analysis.nested_sitemaps {
                    if Url::parse(nested_url).is_ok()
                        && let Some(mut nested_response) = fetch_with_ua(nested_url).await
                        && nested_response.status_code() == 200
                        && let Ok(nested_xml) = nested_response.text().await
                        && let Ok(nested_analysis) =
                            sitemap::parse_sitemap(&nested_xml, &root_origin)
                    {
                        total_urls += nested_analysis.url_count;
                        urls_with_lastmod += nested_analysis.statistics.urls_with_lastmod;
                        urls_with_priority += nested_analysis.statistics.urls_with_priority;
                        priority_sum += nested_analysis.statistics.avg_priority
                            * nested_analysis.statistics.urls_with_priority as f32;
                        // Collect some sample URLs from nested sitemaps
                        all_sample_urls.extend(nested_analysis.url_entries.iter().take(3).cloned());
                    }
                }

                // Update analysis with aggregated data
                if total_urls > 0 {
                    analysis.url_count = total_urls;
                    analysis.statistics.total_urls = total_urls;
                    analysis.statistics.urls_with_lastmod = urls_with_lastmod;
                    analysis.statistics.urls_with_priority = urls_with_priority;
                    analysis.statistics.avg_priority = if urls_with_priority > 0 {
                        priority_sum / urls_with_priority as f32
                    } else {
                        0.0
                    };
                    analysis.url_entries = all_sample_urls;
                }
            }

            // Convert to Worker's SitemapStatus format
            let sitemap_type = match analysis.sitemap_type {
                sitemap::SitemapType::Standard => "standard",
                sitemap::SitemapType::Index => "index",
                sitemap::SitemapType::Unknown => "unknown",
            };

            // Take up to 10 sample URLs for display
            let sample_urls: Vec<SitemapUrlEntry> = analysis
                .url_entries
                .iter()
                .take(10)
                .map(|u| SitemapUrlEntry {
                    loc: u.loc.clone(),
                    lastmod: u.lastmod.clone(),
                    changefreq: u.changefreq.clone(),
                    priority: u.priority,
                })
                .collect();

            return Some(SitemapStatus {
                found: true,
                sitemap_type: sitemap_type.to_string(),
                url_count: analysis.url_count,
                last_modified: analysis.last_modified,
                statistics: SitemapStats {
                    total_urls: analysis.statistics.total_urls,
                    urls_with_lastmod: analysis.statistics.urls_with_lastmod,
                    urls_with_priority: analysis.statistics.urls_with_priority,
                    avg_priority: analysis.statistics.avg_priority,
                    content_types: analysis.statistics.content_types,
                },
                nested_sitemaps: analysis.nested_sitemaps,
                sample_urls,
                issues: analysis.issues,
                recommendations: analysis.recommendations,
            });
        }
    }

    // If no sitemap from robots.txt, try default location
    let default_sitemap = format!("{}/sitemap.xml", root_origin);
    console_log!("[Sitemap] Trying default location: {}", default_sitemap);
    if Url::parse(&default_sitemap).is_ok()
        && let Some(mut response) = fetch_with_ua(&default_sitemap).await
        && response.status_code() == 200
        && let Ok(xml_content) = response.text().await
        && let Ok(mut analysis) = sitemap::parse_sitemap(&xml_content, &root_origin)
    {
        // If this is a sitemap index with nested sitemaps, fetch them to get accurate counts
        if !analysis.nested_sitemaps.is_empty() {
            let mut total_urls = 0;
            let mut urls_with_lastmod = 0;
            let mut urls_with_priority = 0;
            let mut priority_sum = 0.0;
            let mut all_sample_urls = Vec::new();

            for nested_url in &analysis.nested_sitemaps {
                if Url::parse(nested_url).is_ok()
                    && let Some(mut nested_response) = fetch_with_ua(nested_url).await
                    && nested_response.status_code() == 200
                    && let Ok(nested_xml) = nested_response.text().await
                    && let Ok(nested_analysis) = sitemap::parse_sitemap(&nested_xml, &root_origin)
                {
                    total_urls += nested_analysis.url_count;
                    urls_with_lastmod += nested_analysis.statistics.urls_with_lastmod;
                    urls_with_priority += nested_analysis.statistics.urls_with_priority;
                    priority_sum += nested_analysis.statistics.avg_priority
                        * nested_analysis.statistics.urls_with_priority as f32;
                    // Collect some sample URLs from nested sitemaps
                    all_sample_urls.extend(nested_analysis.url_entries.iter().take(3).cloned());
                }
            }

            // Update analysis with aggregated data
            if total_urls > 0 {
                analysis.url_count = total_urls;
                analysis.statistics.total_urls = total_urls;
                analysis.statistics.urls_with_lastmod = urls_with_lastmod;
                analysis.statistics.urls_with_priority = urls_with_priority;
                analysis.statistics.avg_priority = if urls_with_priority > 0 {
                    priority_sum / urls_with_priority as f32
                } else {
                    0.0
                };
                analysis.url_entries = all_sample_urls;
            }
        }

        let sitemap_type = match analysis.sitemap_type {
            sitemap::SitemapType::Standard => "standard",
            sitemap::SitemapType::Index => "index",
            sitemap::SitemapType::Unknown => "unknown",
        };

        let sample_urls: Vec<SitemapUrlEntry> = analysis
            .url_entries
            .iter()
            .take(10)
            .map(|u| SitemapUrlEntry {
                loc: u.loc.clone(),
                lastmod: u.lastmod.clone(),
                changefreq: u.changefreq.clone(),
                priority: u.priority,
            })
            .collect();

        return Some(SitemapStatus {
            found: true,
            sitemap_type: sitemap_type.to_string(),
            url_count: analysis.url_count,
            last_modified: analysis.last_modified,
            statistics: SitemapStats {
                total_urls: analysis.statistics.total_urls,
                urls_with_lastmod: analysis.statistics.urls_with_lastmod,
                urls_with_priority: analysis.statistics.urls_with_priority,
                avg_priority: analysis.statistics.avg_priority,
                content_types: analysis.statistics.content_types,
            },
            nested_sitemaps: analysis.nested_sitemaps,
            sample_urls,
            issues: analysis.issues,
            recommendations: analysis.recommendations,
        });
    }

    // Not found
    Some(SitemapStatus {
        found: false,
        sitemap_type: "unknown".to_string(),
        url_count: 0,
        last_modified: None,
        statistics: SitemapStats {
            total_urls: 0,
            urls_with_lastmod: 0,
            urls_with_priority: 0,
            avg_priority: 0.0,
            content_types: std::collections::HashMap::new(),
        },
        nested_sitemaps: Vec::new(),
        sample_urls: Vec::new(),
        issues: vec!["No sitemap found".to_string()],
        recommendations: vec![
            "Create a sitemap.xml file to help crawlers discover your content".to_string(),
        ],
    })
}

async fn parse_security_txt(base_url: &str) -> Option<SecurityTxtInfo> {
    let url = format!("{}/.well-known/security.txt", base_url);
    Url::parse(&url).ok()?;
    let mut response = fetch_with_ua(&url).await?;

    if response.status_code() != 200 {
        return None;
    }

    let content = match response.text().await {
        Ok(text) => text,
        Err(_) => return None,
    };

    let mut contacts = Vec::new();
    let mut expires = None;
    let mut encryption = Vec::new();
    let mut acknowledgments = Vec::new();
    let mut preferred_languages = None;
    let mut policy = Vec::new();
    let mut hiring = Vec::new();
    let mut canonical = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().to_string();
            if value.is_empty() {
                continue;
            }
            match key.to_lowercase().as_str() {
                "contact" => contacts.push(value),
                "expires" => expires = Some(value),
                "encryption" => encryption.push(value),
                "acknowledgments" | "acknowledgements" => acknowledgments.push(value),
                "preferred-languages" => preferred_languages = Some(value),
                "policy" => policy.push(value),
                "hiring" => hiring.push(value),
                "canonical" => canonical.push(value),
                _ => {}
            }
        }
    }

    // Check if expired
    let is_expired = expires.as_ref().is_some_and(|exp| {
        // Try to parse ISO 8601 date
        if let Some(date_part) = exp.split('T').next() {
            // Simple comparison: if date string is before current date
            date_part < "2026-04-01"
        } else {
            false
        }
    });

    Some(SecurityTxtInfo {
        contacts,
        expires,
        encryption,
        acknowledgments,
        preferred_languages,
        policy,
        hiring,
        canonical,
        is_expired,
    })
}

async fn check_llms_txt(base_url: &str) -> Option<LlmsTxtStatus> {
    let url = format!("{}/llms.txt", base_url);
    console_log!("[AI Readiness] Checking llms.txt: {}", url);

    Url::parse(&url).ok()?;
    let mut response = fetch_with_ua(&url).await?;

    if response.status_code() != 200 {
        return Some(LlmsTxtStatus {
            found: false,
            title: None,
            description: None,
            section_count: 0,
            link_count: 0,
            has_optional_section: false,
            sections: Vec::new(),
        });
    }

    let content = match response.text().await {
        Ok(text) => text,
        Err(_) => return None,
    };

    // Parse the markdown structure
    let mut title = None;
    let mut description = None;
    let mut sections = Vec::new();
    let mut link_count = 0;
    let mut has_optional_section = false;
    let mut in_blockquote = false;
    let mut blockquote_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // H1 = title (first one only)
        if trimmed.starts_with("# ") && title.is_none() {
            title = Some(trimmed.trim_start_matches("# ").to_string());
            continue;
        }

        // H2 = section header
        if trimmed.starts_with("## ") {
            let section_name = trimmed.trim_start_matches("## ").to_string();
            if section_name.to_lowercase() == "optional" {
                has_optional_section = true;
            }
            sections.push(section_name);
            continue;
        }

        // Blockquote = description (first one only)
        if trimmed.starts_with('>') {
            in_blockquote = true;
            blockquote_lines.push(trimmed.trim_start_matches('>').trim().to_string());
            continue;
        } else if in_blockquote {
            in_blockquote = false;
            if description.is_none() {
                description = Some(blockquote_lines.join(" "));
            }
            blockquote_lines.clear();
        }

        // Count markdown links
        let mut remaining = trimmed;
        while let Some(start) = remaining.find("](") {
            if remaining[..start].rfind('[').is_some() {
                link_count += 1;
            }
            remaining = &remaining[start + 2..];
        }
    }

    // Handle blockquote at end of file
    if in_blockquote && description.is_none() && !blockquote_lines.is_empty() {
        description = Some(blockquote_lines.join(" "));
    }

    Some(LlmsTxtStatus {
        found: true,
        title,
        description,
        section_count: sections.len(),
        link_count,
        has_optional_section,
        sections,
    })
}

fn build_semantic_html_status(html: &str) -> SemanticHtmlStatus {
    use htmlens_core::ai_readiness::semantic_html;
    let analysis = semantic_html::analyze_semantic_html(html);

    // Convert to our simplified API format
    let label_percentage = if analysis.forms.total_inputs > 0 {
        (analysis.forms.labeled_inputs as f32 / analysis.forms.total_inputs as f32 * 100.0) as u32
    } else {
        100
    };

    let alt_percentage = if analysis.images.total_images > 0 {
        (analysis.images.images_with_alt as f32 / analysis.images.total_images as f32 * 100.0)
            as u32
    } else {
        100
    };

    SemanticHtmlStatus {
        landmarks: LandmarksInfo {
            has_main: analysis.landmarks.has_main,
            main_count: analysis.landmarks.main_count,
            has_single_main: analysis.landmarks.has_single_main,
            invalid_main_nesting_count: analysis.landmarks.invalid_main_nesting_count,
            has_navigation: analysis.landmarks.has_navigation,
            has_header: analysis.landmarks.has_header,
            has_footer: analysis.landmarks.has_footer,
            article_count: analysis.landmarks.article_count,
            section_count: analysis.landmarks.section_count,
            aside_count: analysis.landmarks.aside_count,
        },
        headings: HeadingsInfo {
            has_single_h1: analysis.headings.has_single_h1,
            proper_hierarchy: analysis.headings.proper_hierarchy,
            distribution: analysis.headings.distribution,
            hierarchy_issues: analysis.headings.hierarchy_issues,
        },
        sections: SectionsInfo {
            sections_without_headings: analysis.sections.sections_without_headings,
            articles_without_headings: analysis.sections.articles_without_headings,
        },
        figures: FiguresInfo {
            total_figures: analysis.figures.total_figures,
            figures_with_captions: analysis.figures.figures_with_captions,
            figures_without_captions: analysis.figures.figures_without_captions,
        },
        links: LinksInfo {
            total_links: analysis.links.total_links,
            links_without_names: analysis.links.links_without_names,
            generic_text_links: analysis.links.generic_text_links,
        },
        element_usage: ElementUsageInfo {
            div_count: analysis.element_usage.div_count,
            semantic_container_count: analysis.element_usage.semantic_container_count,
        },
        aria: AriaInfo {
            labeled_elements: analysis.aria.labeled_elements,
            described_elements: analysis.aria.described_elements,
            live_regions: analysis.aria.live_regions,
            interactive_roles: analysis.aria.interactive_roles,
            misuse_warnings: analysis.aria.misuse_warnings,
        },
        forms: FormsInfo {
            form_count: analysis.forms.form_count,
            total_inputs: analysis.forms.total_inputs,
            labeled_inputs: analysis.forms.labeled_inputs,
            label_percentage,
            forms_with_fieldsets: analysis.forms.forms_with_fieldsets,
            required_fields_marked: analysis.forms.required_fields_marked,
        },
        images: ImagesInfo {
            total_images: analysis.images.total_images,
            images_with_alt: analysis.images.images_with_alt,
            decorative_images: analysis.images.decorative_images,
            images_missing_alt: analysis.images.images_missing_alt,
            alt_percentage,
        },
        issues: analysis.issues,
        recommendations: analysis.recommendations,
    }
}

// ============================================================
// New informational helpers (Phases 2, 4, 5)
// All purely informational — no scoring/issues mutation.
// ============================================================

/// Parse an RFC 8288 Link header. Multiple links may appear comma-separated;
/// each entry is `<target>; rel="..."`. Relation values may be
/// space-separated; we expand them so each (target, rel) pair is its own row.
fn parse_link_header(raw: &str) -> LinkHeaderInfo {
    let mut links: Vec<ParsedLink> = Vec::new();
    // Naive split on commas that are not inside <>. Good enough since
    // targets are always wrapped in <...>.
    let mut depth = 0;
    let mut buf = String::new();
    let mut entries: Vec<String> = Vec::new();
    for ch in raw.chars() {
        match ch {
            '<' => {
                depth += 1;
                buf.push(ch);
            }
            '>' => {
                if depth > 0 {
                    depth -= 1;
                }
                buf.push(ch);
            }
            ',' if depth == 0 => {
                entries.push(std::mem::take(&mut buf));
            }
            _ => buf.push(ch),
        }
    }
    if !buf.trim().is_empty() {
        entries.push(buf);
    }
    for entry in entries {
        let entry = entry.trim();
        let target_start = entry.find('<');
        let target_end = entry.find('>');
        if let (Some(s), Some(e)) = (target_start, target_end) {
            if e <= s {
                continue;
            }
            let target = entry[s + 1..e].trim().to_string();
            let rest = &entry[e + 1..];
            // Find rel= parameter (case-insensitive).
            let mut rel_value: Option<String> = None;
            for param in rest.split(';') {
                let p = param.trim();
                let pl = p.to_lowercase();
                if let Some(v) = pl.strip_prefix("rel=") {
                    let v = v.trim().trim_matches('"').to_string();
                    rel_value = Some(v);
                    break;
                }
            }
            let rels = rel_value.unwrap_or_default();
            if rels.is_empty() {
                links.push(ParsedLink {
                    target: target.clone(),
                    rel: String::new(),
                });
            } else {
                for r in rels.split_whitespace() {
                    links.push(ParsedLink {
                        target: target.clone(),
                        rel: r.to_string(),
                    });
                }
            }
        }
    }
    let agent_useful_rels = [
        "api-catalog",
        "service-doc",
        "service-desc",
        "service-meta",
        "status",
        "terms-of-service",
        "describedby",
        "license",
        "author",
    ];
    let agent_useful = links
        .iter()
        .filter(|l| {
            agent_useful_rels
                .iter()
                .any(|r| l.rel.eq_ignore_ascii_case(r))
        })
        .cloned()
        .collect();
    LinkHeaderInfo {
        raw: raw.to_string(),
        links,
        agent_useful,
    }
}

/// Fetch the homepage with `Accept: text/markdown` and inspect the response.
async fn check_markdown_negotiation(url: &str) -> Option<MarkdownNegotiationInfo> {
    let response = fetch_with_ua_opts(url, Some("text/markdown"), CACHE_TTL_DEFAULT).await?;
    let status = response.status_code();
    let content_type = response
        .headers()
        .get("Content-Type")
        .ok()
        .flatten()
        .unwrap_or_default();
    let supported = status == 200 && content_type.to_lowercase().contains("text/markdown");
    Some(MarkdownNegotiationInfo {
        supported,
        status,
        content_type,
    })
}

/// Parse `/.well-known/oauth-authorization-server` (RFC 8414).
async fn check_oauth_authorization_server(base_url: &str) -> Option<OauthMetadataInfo> {
    let url = format!("{}/.well-known/oauth-authorization-server", base_url);
    let mut response = fetch_with_ua(&url).await?;
    let text = response.text().await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let issuer = json
        .get("issuer")
        .and_then(|v| v.as_str())
        .map(String::from);
    let authorization_endpoint = json
        .get("authorization_endpoint")
        .and_then(|v| v.as_str())
        .map(String::from);
    let token_endpoint = json
        .get("token_endpoint")
        .and_then(|v| v.as_str())
        .map(String::from);
    let scopes_supported = json
        .get("scopes_supported")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Some(OauthMetadataInfo {
        valid: true,
        issuer,
        authorization_endpoint,
        token_endpoint,
        scopes_supported,
    })
}

/// Parse `/.well-known/oauth-protected-resource` (RFC 9728 draft) and merge
/// with the `WWW-Authenticate` header captured from the homepage.
async fn check_oauth_protected_resource(
    base_url: &str,
    www_authenticate: &str,
) -> Option<OauthProtectedResourceInfo> {
    let url = format!("{}/.well-known/oauth-protected-resource", base_url);
    let www_auth_opt = if www_authenticate.is_empty() {
        None
    } else {
        Some(www_authenticate.to_string())
    };
    if let Some(mut response) = fetch_with_ua(&url).await
        && response.status_code() == 200
            && let Ok(text) = response.text().await
                && let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    let resource = json
                        .get("resource")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let authorization_servers = json
                        .get("authorization_servers")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    let scopes_supported = json
                        .get("scopes_supported")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    return Some(OauthProtectedResourceInfo {
                        found: true,
                        resource,
                        authorization_servers,
                        scopes_supported,
                        www_authenticate: www_auth_opt,
                    });
                }
    // Fall back to header-only signal.
    www_auth_opt.as_ref()?;
    Some(OauthProtectedResourceInfo {
        found: false,
        resource: None,
        authorization_servers: Vec::new(),
        scopes_supported: Vec::new(),
        www_authenticate: www_auth_opt,
    })
}

/// Validate `/.well-known/api-catalog` (Linkset, RFC 9264).
async fn check_api_catalog(base_url: &str) -> Option<ApiCatalogInfo> {
    let url = format!("{}/.well-known/api-catalog", base_url);
    let mut response = fetch_with_ua_opts(
        &url,
        Some("application/linkset+json, application/json"),
        CACHE_TTL_DEFAULT,
    )
    .await?;
    let text = response.text().await.ok()?;
    let mut issues = Vec::new();
    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            issues.push(format!("Invalid JSON: {}", e));
            return Some(ApiCatalogInfo {
                valid: false,
                entry_count: 0,
                entries: Vec::new(),
                issues,
            });
        }
    };
    let linkset = json
        .get("linkset")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut entries: Vec<ApiCatalogEntry> = Vec::new();
    for item in &linkset {
        let anchor = item
            .get("anchor")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let mut relations: Vec<String> = Vec::new();
        if let Some(obj) = item.as_object() {
            for key in obj.keys() {
                if key != "anchor" {
                    relations.push(key.clone());
                }
            }
        }
        entries.push(ApiCatalogEntry { anchor, relations });
    }
    if entries.is_empty() {
        issues.push("Linkset is empty".to_string());
    }
    Some(ApiCatalogInfo {
        valid: issues.is_empty(),
        entry_count: entries.len(),
        entries,
        issues,
    })
}

/// Parse an MCP server card. Tries both `server-card.json` and
/// `server-cards.json`.
async fn check_mcp_server_card(base_url: &str) -> Option<McpServerCardInfo> {
    for path in [
        "/.well-known/mcp/server-card.json",
        "/.well-known/mcp/server-cards.json",
    ] {
        let url = format!("{}{}", base_url, path);
        if let Some(mut response) = fetch_with_ua(&url).await
            && response.status_code() == 200
                && let Ok(text) = response.text().await
                    && let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        // server-cards.json is typically an array; flatten to first entry.
                        let node = if json.is_array() {
                            json.as_array()
                                .and_then(|a| a.first().cloned())
                                .unwrap_or(json)
                        } else {
                            json
                        };
                        let name = node.get("name").and_then(|v| v.as_str()).map(String::from);
                        let version = node
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let transport = node
                            .get("transport")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .or_else(|| {
                                node.get("transports")
                                    .and_then(|v| v.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                            });
                        let endpoint = node
                            .get("endpoint")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .or_else(|| node.get("url").and_then(|v| v.as_str()).map(String::from));
                        return Some(McpServerCardInfo {
                            found: true,
                            path: path.to_string(),
                            name,
                            version,
                            transport,
                            endpoint,
                        });
                    }
    }
    None
}

/// Probe for the http-message-signatures-directory presence (Cloudflare's
/// Web Bot Auth scheme). Just records found / status; the directory itself
/// is JSON and already validated by `check_well_known_files`.
async fn check_web_bot_auth(base_url: &str) -> Option<WebBotAuthInfo> {
    let url = format!("{}/.well-known/http-message-signatures-directory", base_url);
    let response = fetch_with_ua(&url).await?;
    let status = response.status_code();
    let content_type = response
        .headers()
        .get("Content-Type")
        .ok()
        .flatten()
        .unwrap_or_default();
    Some(WebBotAuthInfo {
        found: status == 200,
        status,
        content_type,
    })
}

/// Parse agent skills index. Tries new (`/.well-known/agent-skills/index.json`)
/// then legacy (`/.well-known/skills/index.json`).
async fn check_agent_skills(base_url: &str) -> Option<AgentSkillsInfo> {
    for path in [
        "/.well-known/agent-skills/index.json",
        "/.well-known/skills/index.json",
    ] {
        let url = format!("{}{}", base_url, path);
        if let Some(mut response) = fetch_with_ua(&url).await
            && response.status_code() == 200
                && let Ok(text) = response.text().await
                    && let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        let arr = json
                            .get("skills")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .or_else(|| json.as_array().cloned())
                            .unwrap_or_default();
                        let skills: Vec<AgentSkillEntry> = arr
                            .iter()
                            .filter_map(|s| {
                                let name = s
                                    .get("name")
                                    .or_else(|| s.get("id"))
                                    .and_then(|v| v.as_str())?
                                    .to_string();
                                Some(AgentSkillEntry {
                                    name,
                                    skill_type: s
                                        .get("type")
                                        .and_then(|v| v.as_str())
                                        .map(String::from),
                                    description: s
                                        .get("description")
                                        .and_then(|v| v.as_str())
                                        .map(String::from),
                                    url: s.get("url").and_then(|v| v.as_str()).map(String::from),
                                })
                            })
                            .collect();
                        return Some(AgentSkillsInfo {
                            found: true,
                            path: path.to_string(),
                            skill_count: skills.len(),
                            skills,
                        });
                    }
    }
    None
}

/// Parse `/.well-known/ucp` (Universal Commerce Protocol).
async fn check_ucp(base_url: &str) -> Option<UcpInfo> {
    let url = format!("{}/.well-known/ucp", base_url);
    let mut response = fetch_with_ua(&url).await?;
    if response.status_code() != 200 {
        return None;
    }
    let text = response.text().await.ok()?;
    // UCP is typically JSON; tolerate other text forms.
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
        let services = json
            .get("services")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| {
                        x.get("name")
                            .or_else(|| x.get("type"))
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_default();
        return Some(UcpInfo {
            found: true,
            version,
            services,
        });
    }
    Some(UcpInfo {
        found: true,
        version: None,
        services: Vec::new(),
    })
}

/// Parse `/.well-known/acp.json` (Agentic Commerce Protocol).
async fn check_acp(base_url: &str) -> Option<AcpInfo> {
    let url = format!("{}/.well-known/acp.json", base_url);
    let mut response = fetch_with_ua(&url).await?;
    if response.status_code() != 200 {
        return None;
    }
    let text = response.text().await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let version = json
        .get("version")
        .and_then(|v| v.as_str())
        .map(String::from);
    let api_base_url = json
        .get("apiBaseUrl")
        .or_else(|| json.get("api_base_url"))
        .and_then(|v| v.as_str())
        .map(String::from);
    Some(AcpInfo {
        found: true,
        version,
        api_base_url,
    })
}

/// Probe a handful of common paywall paths for an x402 (HTTP 402 Payment
/// Required) signal. Cached with a short TTL so we don't hammer the origin.
async fn check_x402(base_url: &str) -> Option<X402Info> {
    let probes = ["/", "/api", "/api/agent"];
    let mut results: Vec<X402Probe> = Vec::new();
    let mut detected = false;
    for path in probes {
        let url = format!("{}{}", base_url, path);
        if let Some(response) = fetch_with_ua_opts(&url, None, CACHE_TTL_X402).await {
            let status = response.status_code();
            if status == 402 {
                detected = true;
            }
            results.push(X402Probe {
                path: path.to_string(),
                status,
            });
        }
    }
    if results.is_empty() {
        None
    } else {
        Some(X402Info {
            detected,
            probes: results,
        })
    }
}

fn check_crawler_access(robots_txt: &str, crawler: &str) -> String {
    let lines: Vec<&str> = robots_txt.lines().map(|l| l.trim()).collect();
    let mut current_agent = String::new();
    let mut specific_rules = false;
    let mut is_blocked = false;
    let mut has_disallow = false;

    for line in &lines {
        if line.to_lowercase().starts_with("user-agent:") {
            if let Some(agent) = line.split(':').nth(1) {
                current_agent = agent.trim().to_lowercase();
                if current_agent == crawler.to_lowercase() {
                    specific_rules = true;
                }
            }
        } else if !current_agent.is_empty()
            && (current_agent == crawler.to_lowercase() || current_agent == "*")
            && line.to_lowercase().starts_with("disallow:")
            && let Some(path) = line.split(':').nth(1)
        {
            let path = path.trim();
            has_disallow = true;
            if path == "/" {
                is_blocked = true;
            }
        }
    }

    if is_blocked {
        "blocked".to_string()
    } else if has_disallow && specific_rules {
        "partial".to_string()
    } else if specific_rules {
        "allowed".to_string()
    } else {
        "default".to_string()
    }
}

fn get_crawler_rules(robots_txt: &str, crawler: &str) -> Option<String> {
    let lines: Vec<&str> = robots_txt.lines().map(|l| l.trim()).collect();
    let mut rules = Vec::new();
    let mut in_target_agent = false;

    for line in &lines {
        if line.to_lowercase().starts_with("user-agent:") {
            if in_target_agent {
                break; // Stop when we hit next agent
            }
            if let Some(agent) = line.split(':').nth(1) {
                let agent_lower = agent.trim().to_lowercase();
                if agent_lower == crawler.to_lowercase() {
                    in_target_agent = true;
                    rules.push(line.to_string());
                }
            }
        } else if in_target_agent
            && (line.to_lowercase().starts_with("disallow:")
                || line.to_lowercase().starts_with("allow:")
                || line.to_lowercase().starts_with("crawl-delay:"))
        {
            rules.push(line.to_string());
        }
    }

    if rules.is_empty() {
        // Check for wildcard
        let mut in_wildcard = false;
        for line in &lines {
            if line.to_lowercase().starts_with("user-agent:")
                && let Some(agent) = line.split(':').nth(1)
                && agent.trim() == "*"
            {
                in_wildcard = true;
                rules.push(line.to_string());
            } else if in_wildcard
                && (line.to_lowercase().starts_with("disallow:")
                    || line.to_lowercase().starts_with("allow:"))
            {
                rules.push(line.to_string());
            }
        }
    }

    if rules.is_empty() {
        None
    } else {
        Some(rules.join("; "))
    }
}

fn format_cli_style_markdown(
    url: &str,
    title: &str,
    description: &str,
    jsonld_blocks: &[serde_json::Value],
) -> String {
    let mut md = String::new();

    md.push_str("─────────────────────────────────────────────────────────────\n");
    md.push_str(&format!("# {}\n", title));
    md.push_str("─────────────────────────────────────────────────────────────\n\n");

    if !description.is_empty() {
        md.push_str(&format!("> {}\n\n", description));
    }

    md.push_str(&format!("**URL**: {}\n", url));

    // Extract different entity types
    let mut organizations = Vec::new();
    let mut product_groups = Vec::new();
    let mut products = Vec::new();
    let mut breadcrumbs = Vec::new();
    let mut webpages = Vec::new();
    let mut websites = Vec::new();
    let mut images = Vec::new();
    let mut articles = Vec::new();
    let mut news_articles = Vec::new();
    let mut reviews = Vec::new();
    let mut local_businesses = Vec::new();
    let mut events = Vec::new();
    let mut videos = Vec::new();
    let mut other_entities = Vec::new();

    // First, flatten any @graph structures into individual entities
    let mut all_entities: Vec<&serde_json::Value> = Vec::new();
    for block in jsonld_blocks {
        // Check if this block contains a @graph array
        if let Some(graph) = block.get("@graph").and_then(|g| g.as_array()) {
            // Add each item from the graph
            for item in graph {
                all_entities.push(item);
            }
        } else {
            // Direct entity (no @graph wrapper)
            all_entities.push(block);
        }
    }

    for entity in all_entities {
        let entity_type = entity
            .get("@type")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");
        match entity_type {
            "Organization" => organizations.push(entity),
            "ProductGroup" => product_groups.push(entity),
            "Product" => products.push(entity),
            "BreadcrumbList" => breadcrumbs.push(entity),
            "WebPage" => webpages.push(entity),
            "WebSite" => websites.push(entity),
            "ImageObject" => images.push(entity),
            "Article" | "BlogPosting" => articles.push(entity),
            "NewsArticle" => news_articles.push(entity),
            "Review" => reviews.push(entity),
            "AggregateRating" => reviews.push(entity),
            "LocalBusiness" | "Restaurant" | "Store" => local_businesses.push(entity),
            "Event" => events.push(entity),
            "VideoObject" => videos.push(entity),
            _ => other_entities.push((entity_type, entity)),
        }
    }

    // Count total entities
    let total_entities = organizations.len()
        + product_groups.len()
        + products.len()
        + breadcrumbs.len()
        + webpages.len()
        + websites.len()
        + images.len()
        + articles.len()
        + news_articles.len()
        + reviews.len()
        + local_businesses.len()
        + events.len()
        + videos.len()
        + other_entities.len();

    md.push_str(&format!(
        "**Structured Data Entities Found**: {}\n\n",
        total_entities
    ));

    // Render Organizations
    if !organizations.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("🏢 Organization\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for org in organizations {
            if let Some(name) = org.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• Name             : {}\n", name));
            }
            if let Some(url) = org.get("url").and_then(|u| u.as_str()) {
                md.push_str(&format!("• URL              : {}\n", url));
            }
        }
        md.push('\n');
    }

    // Render Breadcrumbs
    if !breadcrumbs.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("🍞 Breadcrumb Navigation\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for bc in breadcrumbs {
            if let Some(items) = bc.get("itemListElement").and_then(|i| i.as_array()) {
                let crumbs: Vec<String> = items
                    .iter()
                    .filter_map(|item| {
                        let name = item
                            .get("name")
                            .or_else(|| item.get("item").and_then(|i| i.get("name")))
                            .and_then(|n| n.as_str())?;
                        let url = item
                            .get("item")
                            .and_then(|i| {
                                if i.is_string() {
                                    i.as_str()
                                } else {
                                    i.get("@id").and_then(|id| id.as_str())
                                }
                            })
                            .unwrap_or("");
                        Some(if !url.is_empty() {
                            format!("{} ({})", name, url)
                        } else {
                            name.to_string()
                        })
                    })
                    .collect();
                md.push_str(&crumbs.join(" → "));
                md.push('\n');
            }
        }
        md.push('\n');
    }

    // Render ProductGroups
    for pg in &product_groups {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        let pg_name = pg
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("ProductGroup");
        md.push_str(&format!("📦 ProductGroup: {}\n", pg_name));
        md.push_str("─────────────────────────────────────────────────────────────\n");

        if let Some(pg_id) = pg.get("productGroupID").and_then(|i| i.as_str()) {
            md.push_str(&format!("• ProductGroup ID  : {}\n", pg_id));
        }
        if let Some(brand) = pg
            .get("brand")
            .and_then(|b| b.get("name"))
            .and_then(|n| n.as_str())
        {
            md.push_str(&format!("• Brand            : {}\n", brand));
        }

        // variesBy
        if let Some(varies) = pg.get("variesBy") {
            let varies_str = if let Some(arr) = varies.as_array() {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                varies.as_str().unwrap_or("").to_string()
            };
            md.push_str(&format!("• Varies By        : {}\n", varies_str));
        }

        // Variant count
        if let Some(variants) = pg.get("hasVariant") {
            let count = if variants.is_array() {
                variants.as_array().unwrap().len()
            } else {
                1
            };
            md.push_str(&format!("• Total Variants   : {}\n", count));

            // Extract price range and availability from offers
            if let Some(offers) = pg.get("offers").and_then(|o| o.as_array()) {
                let prices: Vec<f64> = offers
                    .iter()
                    .filter_map(|o| o.get("price").and_then(|p| p.as_f64()))
                    .collect();

                if !prices.is_empty() {
                    let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let currency = offers[0]
                        .get("priceCurrency")
                        .and_then(|c| c.as_str())
                        .unwrap_or("€");

                    if (min_price - max_price).abs() < 0.01 {
                        md.push_str(&format!(
                            "• Price Range      : {}{:.2}\n",
                            currency, min_price
                        ));
                    } else {
                        md.push_str(&format!(
                            "• Price Range      : {}{:.2} - {}{:.2}\n",
                            currency, min_price, currency, max_price
                        ));
                    }
                }

                let in_stock = offers
                    .iter()
                    .filter(|o| {
                        o.get("availability")
                            .and_then(|a| a.as_str())
                            .is_some_and(|s| s.contains("InStock"))
                    })
                    .count();
                let out_of_stock = offers
                    .iter()
                    .filter(|o| {
                        o.get("availability")
                            .and_then(|a| a.as_str())
                            .is_some_and(|s| s.contains("OutOfStock"))
                    })
                    .count();

                if in_stock > 0 || out_of_stock > 0 {
                    md.push_str(&format!(
                        "• Availability     : {} InStock / {} OutOfStock\n",
                        in_stock, out_of_stock
                    ));
                }
            }
        }

        md.push('\n');

        // Render variant table if available
        if let Some(variants) = pg.get("hasVariant").and_then(|v| v.as_array()) {
            md.push_str("─────────────────────────────────────────────────────────────\n");
            md.push_str("🧩 Variants\n");
            md.push_str("─────────────────────────────────────────────────────────────\n");

            // Build table header
            let varies_by: Vec<&str> = pg
                .get("variesBy")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();

            md.push_str("| SKU       |");
            for prop in &varies_by {
                md.push_str(&format!(" {} |", prop));
            }
            md.push_str(" Price   | Availability   |\n");

            md.push_str("| --------- |");
            for _ in &varies_by {
                md.push_str(" ---- |");
            }
            md.push_str(" ------- | -------------- |\n");

            // Build table rows
            for variant in variants {
                let sku = variant.get("sku").and_then(|s| s.as_str()).unwrap_or("-");
                md.push_str(&format!("| {} |", sku));

                for prop in &varies_by {
                    let value = variant
                        .get(prop)
                        .or_else(|| variant.get(prop.to_lowercase()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    md.push_str(&format!(" {} |", value));
                }

                // Price
                let offer = variant.get("offers").and_then(|o| {
                    if o.is_array() {
                        o.as_array().unwrap().first()
                    } else {
                        Some(o)
                    }
                });
                let currency = offer
                    .and_then(|o| o.get("priceCurrency"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("€");
                let price = offer
                    .and_then(|o| o.get("price"))
                    .and_then(|p| p.as_f64())
                    .map(|p| format!("{}{:.0}", currency, p))
                    .unwrap_or_else(|| "-".to_string());
                md.push_str(&format!(" {} |", price));

                // Availability
                let avail = offer
                    .and_then(|o| o.get("availability"))
                    .and_then(|a| a.as_str())
                    .map(|a| {
                        if a.contains("InStock") {
                            "✅ InStock"
                        } else {
                            "❌ OutOfStock"
                        }
                    })
                    .unwrap_or("-");
                md.push_str(&format!(" {} |\n", avail));
            }
            md.push('\n');
        }
    }

    // Render standalone Products
    if !products.is_empty() && product_groups.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str(&format!("📦 Products Found: {}\n", products.len()));
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for product in products {
            if let Some(name) = product.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• {}\n", name));
            }
        }
        md.push('\n');
    }

    // Render WebSite info
    if !websites.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("🌐 WebSite\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for site in &websites {
            if let Some(name) = site.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• Name             : {}\n", name));
            }
            if let Some(url) = site.get("url").and_then(|u| u.as_str()) {
                md.push_str(&format!("• URL              : {}\n", url));
            }
            if let Some(desc) = site.get("description").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Description      : {}\n", desc));
            }
            if let Some(lang) = site.get("inLanguage").and_then(|l| l.as_str()) {
                md.push_str(&format!("• Language         : {}\n", lang));
            }
            // Check for search action
            if let Some(actions) = site.get("potentialAction").and_then(|a| a.as_array()) {
                for action in actions {
                    if action.get("@type").and_then(|t| t.as_str()) == Some("SearchAction") {
                        md.push_str("• Search           : ✅ Site search enabled\n");
                    }
                }
            }
        }
        md.push('\n');
    }

    // Render WebPage info
    if !webpages.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("📄 WebPage\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for page in &webpages {
            if let Some(name) = page.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• Name             : {}\n", name));
            }
            if let Some(url) = page.get("url").and_then(|u| u.as_str()) {
                md.push_str(&format!("• URL              : {}\n", url));
            }
            if let Some(desc) = page.get("description").and_then(|d| d.as_str()) {
                let truncated = if desc.len() > 100 {
                    format!("{}...", &desc[..100])
                } else {
                    desc.to_string()
                };
                md.push_str(&format!("• Description      : {}\n", truncated));
            }
            if let Some(lang) = page.get("inLanguage").and_then(|l| l.as_str()) {
                md.push_str(&format!("• Language         : {}\n", lang));
            }
            if let Some(published) = page.get("datePublished").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Published        : {}\n", published));
            }
            if let Some(modified) = page.get("dateModified").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Modified         : {}\n", modified));
            }
        }
        md.push('\n');
    }

    // Render other entities
    if !other_entities.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("📋 Other Entities\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for (entity_type, entity) in other_entities {
            let name = entity.get("name").and_then(|n| n.as_str()).unwrap_or("");
            md.push_str(&format!(
                "• {} {}\n",
                entity_type,
                if !name.is_empty() {
                    format!("- {}", name)
                } else {
                    String::new()
                }
            ));
        }
        md.push('\n');
    }

    // Render Articles & BlogPosts
    if !articles.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("📝 Article / BlogPost\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for article in articles {
            if let Some(headline) = article.get("headline").and_then(|h| h.as_str()) {
                md.push_str(&format!("• Title            : {}\n", headline));
            }
            if let Some(author) = article.get("author") {
                if let Some(name) = author.get("name").and_then(|n| n.as_str()) {
                    md.push_str(&format!("• Author           : {}\n", name));
                } else if let Some(author_str) = author.as_str() {
                    md.push_str(&format!("• Author           : {}\n", author_str));
                }
            }
            if let Some(published) = article.get("datePublished").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Published        : {}\n", published));
            }
            if let Some(modified) = article.get("dateModified").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Modified         : {}\n", modified));
            }
            if let Some(desc) = article.get("description").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Description      : {}\n", desc.trim_end()));
            }
        }
        md.push('\n');
    }

    // Render News Articles
    if !news_articles.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("📰 News Article\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for article in news_articles {
            if let Some(headline) = article.get("headline").and_then(|h| h.as_str()) {
                md.push_str(&format!("• Title            : {}\n", headline));
            }
            if let Some(author) = article.get("author") {
                if let Some(name) = author.get("name").and_then(|n| n.as_str()) {
                    md.push_str(&format!("• Author           : {}\n", name));
                } else if let Some(author_str) = author.as_str() {
                    md.push_str(&format!("• Author           : {}\n", author_str));
                }
            }
            if let Some(published) = article.get("datePublished").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Published        : {}\n", published));
            }
            if let Some(image) = article.get("image") {
                if let Some(url) = image.get("url").and_then(|u| u.as_str()) {
                    md.push_str(&format!("• Image            : {}\n", url));
                } else if let Some(url) = image.as_str() {
                    md.push_str(&format!("• Image            : {}\n", url));
                }
            }
        }
        md.push('\n');
    }

    // Render Reviews & Ratings
    if !reviews.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("⭐ Reviews & Ratings\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for review in reviews {
            if let Some(name) = review.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• Name             : {}\n", name));
            }
            if let Some(rating) = review.get("ratingValue").and_then(|r| r.as_f64()) {
                if let Some(best) = review.get("bestRating").and_then(|b| b.as_f64()) {
                    let stars = "★".repeat(rating as usize)
                        + &"☆".repeat((best as usize) - (rating as usize));
                    md.push_str(&format!(
                        "• Rating           : {} ({}/{})\n",
                        stars, rating, best
                    ));
                } else {
                    md.push_str(&format!("• Rating           : {}\n", rating));
                }
            }
            if let Some(count) = review.get("reviewCount").and_then(|c| c.as_u64()) {
                md.push_str(&format!("• Review Count     : {}\n", count));
            }
        }
        md.push('\n');
    }

    // Render LocalBusiness
    if !local_businesses.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("🏪 Local Business\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for business in local_businesses {
            if let Some(name) = business.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• Name             : {}\n", name));
            }
            if let Some(address) = business.get("address")
                && let Some(street) = address.get("streetAddress").and_then(|s| s.as_str()) {
                    md.push_str(&format!("• Address          : {}", street));
                    if let Some(city) = address.get("addressLocality").and_then(|c| c.as_str()) {
                        md.push_str(&format!(", {}", city));
                    }
                    md.push('\n');
                }
            if let Some(phone) = business.get("telephone").and_then(|p| p.as_str()) {
                md.push_str(&format!("• Phone            : {}\n", phone));
            }
            if let Some(price) = business.get("priceRange").and_then(|p| p.as_str()) {
                md.push_str(&format!("• Price Range      : {}\n", price));
            }
        }
        md.push('\n');
    }

    // Render Events
    if !events.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("🎯 Events\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for event in events {
            if let Some(name) = event.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• Name             : {}\n", name));
            }
            if let Some(start) = event.get("startDate").and_then(|s| s.as_str()) {
                md.push_str(&format!("• Start Date       : {}\n", start));
            }
            if let Some(end) = event.get("endDate").and_then(|e| e.as_str()) {
                md.push_str(&format!("• End Date         : {}\n", end));
            }
            if let Some(location) = event.get("location")
                && let Some(name) = location.get("name").and_then(|n| n.as_str()) {
                    md.push_str(&format!("• Location         : {}\n", name));
                }
            if let Some(status) = event.get("eventStatus").and_then(|s| s.as_str()) {
                md.push_str(&format!("• Status           : {}\n", status));
            }
        }
        md.push('\n');
    }

    // Render Videos
    if !videos.is_empty() {
        md.push_str("─────────────────────────────────────────────────────────────\n");
        md.push_str("🎬 Videos\n");
        md.push_str("─────────────────────────────────────────────────────────────\n");
        for video in videos {
            if let Some(name) = video.get("name").and_then(|n| n.as_str()) {
                md.push_str(&format!("• Title            : {}\n", name));
            }
            if let Some(duration) = video.get("duration").and_then(|d| d.as_str()) {
                md.push_str(&format!("• Duration         : {}\n", duration));
            }
            if let Some(uploaded) = video.get("uploadDate").and_then(|u| u.as_str()) {
                md.push_str(&format!("• Uploaded         : {}\n", uploaded));
            }
            if let Some(url) = video.get("contentUrl").and_then(|u| u.as_str()) {
                md.push_str(&format!("• Content URL      : {}\n", url));
            }
        }
        md.push('\n');
    }

    md.push_str("─────────────────────────────────────────────────────────────\n");
    md.push_str(&format!(
        "📊 Total: {} Structured Data Entities\n",
        total_entities
    ));

    md
}

#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let url = req.url()?;
    console_log!("[Worker] {} {}", req.method(), url.path());

    // CORS headers
    let headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;

    // Handle CORS preflight
    if req.method() == Method::Options {
        return Response::empty().map(|r| r.with_headers(headers));
    }

    // Handle API endpoint
    if url.path() == "/api" {
        // Parse query parameters: url= (required) and nocache=1 (optional)
        let mut target_url_opt: Option<String> = None;
        let mut bypass_cache = false;
        // Also accept Cache-Control: no-cache request header as bypass signal.
        if let Ok(Some(cc)) = req.headers().get("Cache-Control")
            && cc.to_lowercase().contains("no-cache") {
                bypass_cache = true;
            }
        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                "url" => target_url_opt = Some(v.to_string()),
                "nocache" => {
                    if matches!(v.as_ref(), "1" | "true" | "yes") {
                        bypass_cache = true;
                    }
                }
                _ => {}
            }
        }

        if let Some(target_url) = target_url_opt {
            set_bypass_cache(bypass_cache);
            if bypass_cache {
                console_log!("[Worker] Cache bypass requested (nocache=1)");
            }

            // Validate URL for security - only allow HTTP/HTTPS
            let parsed_url = match url::Url::parse(&target_url) {
                Ok(u) => u,
                Err(_) => {
                    headers.set("Content-Type", "application/json")?;
                    let error_json = serde_json::json!({"error": "Invalid URL format"});
                    return Response::ok(error_json.to_string())
                        .map(|r| r.with_headers(headers).with_status(400));
                }
            };

            // Only allow HTTP and HTTPS schemes
            match parsed_url.scheme() {
                "http" | "https" => {}
                scheme => {
                    headers.set("Content-Type", "application/json")?;
                    let error_json = serde_json::json!({"error": format!("Unsupported URL scheme: {}. Only HTTP and HTTPS are allowed.", scheme)});
                    return Response::ok(error_json.to_string())
                        .map(|r| r.with_headers(headers).with_status(400));
                }
            }

            // Block potentially dangerous hosts (SSRF protection)
            if let Some(host) = parsed_url.host_str() {
                let host_lower = host.to_lowercase();
                if host_lower == "localhost" 
                        || host_lower == "127.0.0.1" 
                        || host_lower == "::1"
                        || host_lower == "[::1]"  // IPv6 localhost with brackets
                        || host_lower.starts_with("192.168.")
                        || host_lower.starts_with("10.")
                        || host_lower.starts_with("172.16.")
                        || host_lower.starts_with("172.17.")
                        || host_lower.starts_with("172.18.")
                        || host_lower.starts_with("172.19.")
                        || host_lower.starts_with("172.20.")
                        || host_lower.starts_with("172.21.")
                        || host_lower.starts_with("172.22.")
                        || host_lower.starts_with("172.23.")
                        || host_lower.starts_with("172.24.")
                        || host_lower.starts_with("172.25.")
                        || host_lower.starts_with("172.26.")
                        || host_lower.starts_with("172.27.")
                        || host_lower.starts_with("172.28.")
                        || host_lower.starts_with("172.29.")
                        || host_lower.starts_with("172.30.")
                        || host_lower.starts_with("172.31.")
                        || host_lower.starts_with("169.254.")
                {
                    headers.set("Content-Type", "application/json")?;
                    let error_json = serde_json::json!({"error": "Access to localhost and private IP addresses is not allowed"});
                    return Response::ok(error_json.to_string())
                        .map(|r| r.with_headers(headers).with_status(400));
                }
            }

            console_log!("[Worker] Processing validated URL: {}", target_url);

            // Fetch HTML
            let mut fetch_req = Request::new(&target_url, Method::Get)?;
            fetch_req
                .headers_mut()?
                .set("User-Agent", "HTMLens/0.4.0 (Cloudflare Worker)")?;

            let mut fetch_response = match Fetch::Request(fetch_req).send().await {
                Ok(r) => r,
                Err(e) => {
                    headers.set("Content-Type", "application/json")?;
                    let error_json =
                        serde_json::json!({"error": format!("Failed to fetch: {}", e)});
                    return Response::ok(error_json.to_string())
                        .map(|r| r.with_headers(headers).with_status(500));
                }
            };

            // Capture response headers that several AI-readiness checks reuse,
            // so we don't have to re-fetch the homepage.
            let homepage_link_header = fetch_response
                .headers()
                .get("Link")
                .ok()
                .flatten()
                .unwrap_or_default();
            let homepage_www_auth = fetch_response
                .headers()
                .get("WWW-Authenticate")
                .ok()
                .flatten()
                .unwrap_or_default();
            let homepage_status_code = fetch_response.status_code();
            let homepage_content_type = fetch_response
                .headers()
                .get("Content-Type")
                .ok()
                .flatten()
                .unwrap_or_default();

            let html = fetch_response.text().await?;
            console_log!("[Worker] Fetched {} bytes", html.len());

            // Extract JSON-LD using htmlens-core FIRST (before sanitizing)
            let (jsonld_blocks, jsonld_graph) = match parser::extract_json_ld_blocks(&html) {
                Ok(blocks) => {
                    console_log!("[Worker] Found {} blocks", blocks.len());

                    // Combine blocks into @graph structure
                    let combined = parser::combine_json_ld_blocks(&blocks).unwrap_or_else(|_| {
                        r#"{"@context":"https://schema.org","@graph":[]}"#.to_string()
                    });

                    let graph_value: serde_json::Value = serde_json::from_str(&combined)
                        .unwrap_or_else(
                            |_| serde_json::json!({"@context": "https://schema.org", "@graph": []}),
                        );

                    let blocks_vec: Vec<serde_json::Value> = blocks
                        .into_iter()
                        .filter_map(|b| serde_json::from_str(&b).ok())
                        .collect();

                    (blocks_vec, graph_value)
                }
                Err(_) => (
                    vec![],
                    serde_json::json!({"@context": "https://schema.org", "@graph": []}),
                ),
            };

            // Convert HTML to Markdown using htmlens-core
            // Sanitizes HTML (removes scripts, styles) and converts to markdown
            let page_markdown = parser::html_to_markdown(&html);
            console_log!(
                "[Worker] Page markdown generated: {} bytes",
                page_markdown.len()
            );

            // Extract metadata
            let title = extract_title(&html);
            let description = extract_description(&html);

            // Build graph nodes
            let nodes: Vec<GraphNode> = jsonld_blocks
                .iter()
                .enumerate()
                .map(|(idx, block)| {
                    let node_type = block
                        .get("@type")
                        .and_then(|t| t.as_str())
                        .map(|s| vec![s.to_string()])
                        .unwrap_or_else(|| vec!["Unknown".to_string()]);

                    let name = block
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string());

                    let mut properties = std::collections::HashMap::new();
                    if let Some(name) = name {
                        properties.insert("name".to_string(), serde_json::Value::String(name));
                    }

                    GraphNode {
                        id: format!("jsonld-{}", idx),
                        types: node_type,
                        properties,
                    }
                })
                .collect();

            // Build CLI-style markdown
            let markdown =
                format_cli_style_markdown(&target_url, &title, &description, &jsonld_blocks);

            // Check AI readiness
            let ai_readiness = check_ai_readiness(
                &target_url,
                HomepageSignals {
                    link_header: homepage_link_header,
                    www_authenticate: homepage_www_auth,
                    status_code: homepage_status_code,
                    content_type: homepage_content_type,
                    html: &html,
                },
            )
            .await;

            let response_data = ApiResponse {
                url: target_url,
                title,
                description,
                graph: GraphData {
                    nodes,
                    edges: vec![],
                },
                jsonld: jsonld_blocks.clone(),
                jsonld_graph,  // Combined @graph structure
                markdown,      // CLI-style product tables
                page_markdown, // HTML to Markdown conversion
                meta: MetaData {
                    html_length: html.len(),
                    jsonld_count: jsonld_blocks.len(),
                    wasm_status: "rust".to_string(),
                },
                ai_readiness,
            };

            headers.set("Content-Type", "application/json")?;
            return Response::ok(serde_json::to_string_pretty(&response_data)?)
                .map(|r| r.with_headers(headers));
        }

        // No URL parameter - return error
        headers.set("Content-Type", "application/json")?;
        let error_json = serde_json::json!({"error": "Missing 'url' query parameter"});
        return Response::ok(error_json.to_string())
            .map(|r| r.with_headers(headers).with_status(400));
    }

    // Serve frontend HTML for root path (including with URL params for sharing)
    if url.path() == "/" {
        let origin_str = url.origin().unicode_serialization();
        let html = FRONTEND_HTML.replace("${origin}", &origin_str);
        headers.set("Content-Type", "text/html;charset=UTF-8")?;
        return Response::ok(html).map(|r| r.with_headers(headers));
    }

    // Health check
    if url.path() == "/health" {
        headers.set("Content-Type", "application/json")?;
        let health = serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION")
        });
        return Response::ok(health.to_string()).map(|r| r.with_headers(headers));
    }

    Response::error("Not Found", 404)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_semantic_html_response_requires_successful_html() {
        let html = "<!doctype html><html><body><main></main></body></html>";

        assert!(is_semantic_html_response(
            200,
            "text/html; charset=UTF-8",
            html
        ));
        assert!(is_semantic_html_response(
            200,
            "Application/XHTML+XML",
            html
        ));
        assert!(is_semantic_html_response(200, "", html));
        assert!(!is_semantic_html_response(500, "text/html", html));
        assert!(!is_semantic_html_response(
            200,
            "application/json",
            r#"{"error":"not found"}"#
        ));
        assert!(!is_semantic_html_response(
            200,
            "",
            r#"{"error":"not found"}"#
        ));
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse {
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            description: "Test desc".to_string(),
            graph: GraphData {
                nodes: vec![],
                edges: vec![],
            },
            jsonld: vec![],
            jsonld_graph: serde_json::json!({"@context": "https://schema.org", "@graph": []}),
            markdown: "# Test".to_string(),
            page_markdown: "Test content".to_string(),
            meta: MetaData {
                html_length: 100,
                jsonld_count: 0,
                wasm_status: "rust".to_string(),
            },
            ai_readiness: AiReadinessData {
                well_known: WellKnownChecks {
                    ai_plugin_json: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    mcp_json: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    openid_configuration: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    security_txt: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    apple_app_site_association: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    assetlinks: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    llms_txt: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    oauth_authorization_server: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    oauth_protected_resource: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    http_message_signatures_directory: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    api_catalog: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    mcp_server_card: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    mcp_server_cards: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    agent_skills_index: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    agent_skills_legacy: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    ucp: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                    acp_json: FileStatus {
                        found: false,
                        status: 404,
                        valid: None,
                    },
                },
                ai_plugin: None,
                mcp: None,
                openapi: None,
                robots_txt: None,
                sitemap: None,
                semantic_html: None,
                security_txt: None,
                llms_txt: None,
                link_headers: None,
                markdown_negotiation: None,
                oauth_authorization_server: None,
                oauth_protected_resource: None,
                api_catalog: None,
                mcp_server_card: None,
                web_bot_auth: None,
                agent_skills: None,
                commerce: CommerceChecks::default(),
            },
        };

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
    }
}
