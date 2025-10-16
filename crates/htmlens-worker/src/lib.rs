//! Cloudflare Worker for htmlens
//! 
//! Pure Rust implementation that serves a web UI and extracts JSON-LD using htmlens-core

use worker::*;
use serde::Serialize;
use htmlens_core::{parser, GraphNode};

#[derive(Serialize)]
struct ApiResponse {
    url: String,
    title: String,
    description: String,
    graph: GraphData,
    jsonld: Vec<serde_json::Value>,
    #[serde(rename = "jsonldGraph")]
    jsonld_graph: serde_json::Value,  // Combined JSON-LD as @graph
    markdown: String,  // CLI-style formatted markdown with product tables
    #[serde(rename = "pageMarkdown")]
    page_markdown: String,  // CF AI converted HTML page content
    meta: MetaData,
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

// Frontend HTML will be included as a separate file
const FRONTEND_HTML: &str = include_str!("frontend.html");

fn extract_title(html: &str) -> String {
    // Simple regex-based title extraction
    if let Some(start) = html.find("<title")
        && let Some(content_start) = html[start..].find('>') {
            let content_start = start + content_start + 1;
            if let Some(end) = html[content_start..].find("</title>") {
                return html[content_start..content_start + end].trim().to_string();
            }
        }
    "Unknown Title".to_string()
}

fn extract_description(html: &str) -> String {
    // Simple meta description extraction
    if let Some(start) = html.find(r#"name="description""#).or_else(|| html.find(r#"name='description'"#))
        && let Some(content) = html[start..].find("content=") {
            let content_start = start + content + 8;
            let quote = html.chars().nth(content_start).unwrap_or('"');
            if let Some(end) = html[content_start + 1..].find(quote) {
                return html[content_start + 1..content_start + 1 + end].to_string();
            }
        }
    String::new()
}

fn format_cli_style_markdown(url: &str, title: &str, description: &str, jsonld_blocks: &[serde_json::Value]) -> String {
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
        let entity_type = block.get("@type").and_then(|t| t.as_str()).unwrap_or("Unknown");
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
                let crumbs: Vec<String> = items.iter()
                    .filter_map(|item| {
                        let name = item.get("name").or_else(|| item.get("item").and_then(|i| i.get("name")))
                            .and_then(|n| n.as_str())?;
                        let url = item.get("item").and_then(|i| {
                            if i.is_string() { i.as_str() } else { i.get("@id").and_then(|id| id.as_str()) }
                        }).unwrap_or("");
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
        let pg_name = pg.get("name").and_then(|n| n.as_str()).unwrap_or("ProductGroup");
        md.push_str(&format!("📦 ProductGroup: {}\n", pg_name));
        md.push_str("─────────────────────────────────────────────────────────────\n");
        
        if let Some(pg_id) = pg.get("productGroupID").and_then(|i| i.as_str()) {
            md.push_str(&format!("• ProductGroup ID  : {}\n", pg_id));
        }
        if let Some(brand) = pg.get("brand").and_then(|b| b.get("name")).and_then(|n| n.as_str()) {
            md.push_str(&format!("• Brand            : {}\n", brand));
        }
        
        // variesBy
        if let Some(varies) = pg.get("variesBy") {
            let varies_str = if let Some(arr) = varies.as_array() {
                arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")
            } else {
                varies.as_str().unwrap_or("").to_string()
            };
            md.push_str(&format!("• Varies By        : {}\n", varies_str));
        }
        
        // Variant count
        if let Some(variants) = pg.get("hasVariant") {
            let count = if variants.is_array() { variants.as_array().unwrap().len() } else { 1 };
            md.push_str(&format!("• Total Variants   : {}\n", count));
            
            // Extract price range and availability from offers
            if let Some(offers) = pg.get("offers").and_then(|o| o.as_array()) {
                let prices: Vec<f64> = offers.iter()
                    .filter_map(|o| o.get("price").and_then(|p| p.as_f64()))
                    .collect();
                
                if !prices.is_empty() {
                    let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let currency = offers[0].get("priceCurrency").and_then(|c| c.as_str()).unwrap_or("€");
                    
                    if (min_price - max_price).abs() < 0.01 {
                        md.push_str(&format!("• Price Range      : {}{:.2}\n", currency, min_price));
                    } else {
                        md.push_str(&format!("• Price Range      : {}{:.2} - {}{:.2}\n", currency, min_price, currency, max_price));
                    }
                }
                
                let in_stock = offers.iter()
                    .filter(|o| o.get("availability").and_then(|a| a.as_str()).is_some_and(|s| s.contains("InStock")))
                    .count();
                let out_of_stock = offers.iter()
                    .filter(|o| o.get("availability").and_then(|a| a.as_str()).is_some_and(|s| s.contains("OutOfStock")))
                    .count();
                
                if in_stock > 0 || out_of_stock > 0 {
                    md.push_str(&format!("• Availability     : {} InStock / {} OutOfStock\n", in_stock, out_of_stock));
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
            let varies_by: Vec<&str> = pg.get("variesBy")
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
                    let value = variant.get(prop)
                        .or_else(|| variant.get(prop.to_lowercase()))
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    md.push_str(&format!(" {} |", value));
                }
                
                // Price
                let offer = variant.get("offers")
                    .and_then(|o| if o.is_array() { o.as_array().unwrap().first() } else { Some(o) });
                let currency = offer.and_then(|o| o.get("priceCurrency")).and_then(|c| c.as_str()).unwrap_or("€");
                let price = offer.and_then(|o| o.get("price")).and_then(|p| p.as_f64())
                    .map(|p| format!("{}{:.0}", currency, p))
                    .unwrap_or_else(|| "-".to_string());
                md.push_str(&format!(" {} |", price));
                
                // Availability
                let avail = offer.and_then(|o| o.get("availability")).and_then(|a| a.as_str())
                    .map(|a| if a.contains("InStock") { "✅ InStock" } else { "❌ OutOfStock" })
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
            md.push_str(&format!("• {} {}\n", entity_type, if !name.is_empty() { format!("- {}", name) } else { String::new() }));
        }
        md.push('\n');
    }
    
    md.push_str("─────────────────────────────────────────────────────────────\n");
    md.push_str(&format!("📊 Total: {} JSON-LD blocks\n", jsonld_blocks.len()));
    
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
    
    // Handle root path
    if url.path() == "/" {
        // Check for URL query parameter
        if let Some(query) = url.query()
            && let Some(target_url_encoded) = query.strip_prefix("url=") {
                let target_url = urlencoding::decode(target_url_encoded)
                    .unwrap_or_default()
                    .to_string();
                
                console_log!("[Worker] Processing URL: {}", target_url);
                
                // Fetch HTML
                let mut fetch_req = Request::new(&target_url, Method::Get)?;
                fetch_req.headers_mut()?.set("User-Agent", "HTMLens/0.4.0 (Cloudflare Worker)")?;
                
                let mut fetch_response = match Fetch::Request(fetch_req).send().await {
                    Ok(r) => r,
                    Err(e) => {
                        headers.set("Content-Type", "application/json")?;
                        let error_json = serde_json::json!({"error": format!("Failed to fetch: {}", e)});
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
                            .unwrap_or_else(|_| serde_json::json!({"@context": "https://schema.org", "@graph": []}));
                        
                        let blocks_vec: Vec<serde_json::Value> = blocks.into_iter()
                            .filter_map(|b| serde_json::from_str(&b).ok())
                            .collect();
                        
                        (blocks_vec, graph_value)
                    }
                    Err(_) => (vec![], serde_json::json!({"@context": "https://schema.org", "@graph": []})),
                };
                
                // Convert HTML to Markdown using htmlens-core
                // Sanitizes HTML (removes scripts, styles) and converts to markdown
                let page_markdown = parser::html_to_markdown(&html);
                console_log!("[Worker] Page markdown generated: {} bytes", page_markdown.len());
                
                // Extract metadata
                let title = extract_title(&html);
                let description = extract_description(&html);
                
                // Build graph nodes
                let nodes: Vec<GraphNode> = jsonld_blocks.iter().enumerate()
                    .map(|(idx, block)| {
                        let node_type = block.get("@type")
                            .and_then(|t| t.as_str())
                            .map(|s| vec![s.to_string()])
                            .unwrap_or_else(|| vec!["Unknown".to_string()]);
                        
                        let name = block.get("name")
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
                let markdown = format_cli_style_markdown(&target_url, &title, &description, &jsonld_blocks);
                
                let response_data = ApiResponse {
                    url: target_url,
                    title,
                    description,
                    graph: GraphData { nodes, edges: vec![] },
                    jsonld: jsonld_blocks.clone(),
                    jsonld_graph,  // Combined @graph structure
                    markdown,  // CLI-style product tables
                    page_markdown,  // HTML to Markdown conversion
                    meta: MetaData {
                        html_length: html.len(),
                        jsonld_count: jsonld_blocks.len(),
                        wasm_status: "rust".to_string(),
                    },
                };
                
                headers.set("Content-Type", "application/json")?;
                return Response::ok(serde_json::to_string_pretty(&response_data)?)
                    .map(|r| r.with_headers(headers));
            }
        
        // Serve frontend HTML
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
