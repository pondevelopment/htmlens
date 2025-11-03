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
    forms: FormsInfo,
    images: ImagesInfo,
    issues: Vec<String>,
    recommendations: Vec<String>,
}

#[derive(Serialize)]
struct LandmarksInfo {
    #[serde(rename = "hasMain")]
    has_main: bool,
    #[serde(rename = "hasNavigation")]
    has_navigation: bool,
    #[serde(rename = "hasHeader")]
    has_header: bool,
    #[serde(rename = "hasFooter")]
    has_footer: bool,
    #[serde(rename = "articleCount")]
    article_count: usize,
}

#[derive(Serialize)]
struct HeadingsInfo {
    #[serde(rename = "hasSingleH1")]
    has_single_h1: bool,
    #[serde(rename = "properHierarchy")]
    proper_hierarchy: bool,
    distribution: Vec<usize>,
}

#[derive(Serialize)]
struct FormsInfo {
    #[serde(rename = "totalInputs")]
    total_inputs: usize,
    #[serde(rename = "labeledInputs")]
    labeled_inputs: usize,
    #[serde(rename = "labelPercentage")]
    label_percentage: u32,
}

#[derive(Serialize)]
struct ImagesInfo {
    #[serde(rename = "totalImages")]
    total_images: usize,
    #[serde(rename = "imagesWithAlt")]
    images_with_alt: usize,
    #[serde(rename = "altPercentage")]
    alt_percentage: u32,
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

async fn check_ai_readiness(base_url: &str) -> AiReadinessData {
    let origin = normalize_origin(base_url);

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

    // Check semantic HTML
    let semantic_html = check_semantic_html(base_url).await;

    AiReadinessData {
        well_known,
        ai_plugin,
        mcp,
        openapi,
        robots_txt,
        sitemap,
        semantic_html,
    }
}

async fn check_well_known_files(base_url: &str) -> WellKnownChecks {
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
    };

    for (filename, _) in files {
        let url = format!("{}/.well-known/{}", base_url, filename);
        if let Ok(parsed) = Url::parse(&url)
            && let Ok(mut response) = Fetch::Url(parsed).send().await
        {
            let status = response.status_code();
            let found = status == 200;
            let valid = if found && (filename.ends_with(".json") || filename == "assetlinks.json") {
                response
                    .text()
                    .await
                    .ok()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                    .map(|_| true)
            } else {
                None
            };

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

    results
}

async fn check_ai_plugin(base_url: &str) -> Option<AiPluginStatus> {
    let url = format!("{}/.well-known/ai-plugin.json", base_url);
    if let Ok(parsed) = Url::parse(&url)
        && let Ok(mut response) = Fetch::Url(parsed).send().await
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
    if let Ok(parsed) = Url::parse(&url)
        && let Ok(mut response) = Fetch::Url(parsed).send().await
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
    if let Ok(parsed) = Url::parse(api_url)
        && let Ok(mut response) = Fetch::Url(parsed).send().await
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
    let parsed = Url::parse(&robots_url).ok()?;

    if let Ok(mut response) = Fetch::Url(parsed).send().await
        && response.status_code() == 200
        && let Ok(text) = response.text().await
    {
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

        return Some(RobotsTxtStatus {
            found: true,
            sitemap_count: sitemaps.len(),
            sitemaps,
            ai_crawlers,
            blocks_all_bots,
            issues,
        });
    }

    Some(RobotsTxtStatus {
        found: false,
        sitemap_count: 0,
        sitemaps: Vec::new(),
        ai_crawlers: Vec::new(),
        blocks_all_bots: false,
        issues: vec!["robots.txt not found".to_string()],
    })
}

async fn check_sitemap(base_url: &str, sitemap_urls: Vec<String>) -> Option<SitemapStatus> {
    use htmlens_core::ai_readiness::sitemap;

    // Extract root domain from URL
    let root_origin = normalize_origin(base_url);
    Url::parse(&root_origin).ok()?;

    // Try sitemap URLs from robots.txt first
    for sitemap_url in &sitemap_urls {
        if let Ok(parsed) = Url::parse(sitemap_url)
            && let Ok(mut response) = Fetch::Url(parsed).send().await
            && response.status_code() == 200
            && let Ok(xml_content) = response.text().await
            && let Ok(analysis) = sitemap::parse_sitemap(&xml_content, &root_origin)
        {
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
    if let Ok(parsed) = Url::parse(&default_sitemap)
        && let Ok(mut response) = Fetch::Url(parsed).send().await
        && response.status_code() == 200
        && let Ok(xml_content) = response.text().await
        && let Ok(analysis) = sitemap::parse_sitemap(&xml_content, &root_origin)
    {
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

async fn check_semantic_html(base_url: &str) -> Option<SemanticHtmlStatus> {
    // Fetch the HTML page
    let url = base_url.trim_end_matches('/');
    let parsed = Url::parse(url).ok()?;
    let mut response = match Fetch::Url(parsed).send().await {
        Ok(r) => r,
        Err(_) => return None,
    };

    if response.status_code() != 200 {
        return None;
    }

    let html = match response.text().await {
        Ok(text) => text,
        Err(_) => return None,
    };

    // Use the semantic_html analyzer from htmlens-core
    use htmlens_core::ai_readiness::semantic_html;
    let analysis = semantic_html::analyze_semantic_html(&html);

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

    Some(SemanticHtmlStatus {
        landmarks: LandmarksInfo {
            has_main: analysis.landmarks.has_main,
            has_navigation: analysis.landmarks.has_navigation,
            has_header: analysis.landmarks.has_header,
            has_footer: analysis.landmarks.has_footer,
            article_count: analysis.landmarks.article_count,
        },
        headings: HeadingsInfo {
            has_single_h1: analysis.headings.has_single_h1,
            proper_hierarchy: analysis.headings.proper_hierarchy,
            distribution: analysis.headings.distribution,
        },
        forms: FormsInfo {
            total_inputs: analysis.forms.total_inputs,
            labeled_inputs: analysis.forms.labeled_inputs,
            label_percentage,
        },
        images: ImagesInfo {
            total_images: analysis.images.total_images,
            images_with_alt: analysis.images.images_with_alt,
            alt_percentage,
        },
        issues: analysis.issues,
        recommendations: analysis.recommendations,
    })
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
    md.push_str(&format!("**JSON-LD Blocks**: {}\n\n", jsonld_blocks.len()));

    // Extract different entity types
    let mut organizations = Vec::new();
    let mut product_groups = Vec::new();
    let mut products = Vec::new();
    let mut breadcrumbs = Vec::new();
    let mut other_entities = Vec::new();

    for block in jsonld_blocks {
        let entity_type = block
            .get("@type")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown");
        match entity_type {
            "Organization" => organizations.push(block),
            "ProductGroup" => product_groups.push(block),
            "Product" => products.push(block),
            "BreadcrumbList" => breadcrumbs.push(block),
            _ => other_entities.push((entity_type, block)),
        }
    }

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

    md.push_str("─────────────────────────────────────────────────────────────\n");
    md.push_str(&format!(
        "📊 Total: {} JSON-LD blocks\n",
        jsonld_blocks.len()
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
        // Check for URL query parameter
        if let Some(query) = url.query()
            && let Some(target_url_encoded) = query.strip_prefix("url=")
        {
            let target_url = urlencoding::decode(target_url_encoded)
                .unwrap_or_default()
                .to_string();

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
            let ai_readiness = check_ai_readiness(&target_url).await;

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
                },
                ai_plugin: None,
                mcp: None,
                openapi: None,
                robots_txt: None,
                sitemap: None,
                semantic_html: None,
            },
        };

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
    }
}
