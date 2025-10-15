use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use json_ld::ReqwestLoader;
use serde_json::Value as JsonValue;
use url::Url;

mod parser;
mod ld_graph;

use ld_graph::{KnowledgeGraph, GraphNode, GraphEdge, GraphBuilder};

const APP_NAME: &str = "htmlens";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, PartialEq)]
enum OutputMode {
    Default,      // Markdown + summaries (default behavior)
    GraphOnly,    // Only condensed graph summary
}

enum InputSource {
    Url(String),
    JsonLd(String),
}

struct CliOptions {
    input: InputSource,
    mode: OutputMode,
    include_data_downloads: bool,
    include_mermaid: bool,
    save_target: Option<PathBuf>,
}

enum CliCommand {
    Run(CliOptions),
    Help,
    Version,
}

fn parse_arguments(args: &[String]) -> Result<CliCommand> {
    if args.is_empty() {
        return Ok(CliCommand::Help);
    }

    let mut url: Option<String> = None;
    let mut mode = OutputMode::Default;
    let mut include_data_downloads = false;
    let mut include_mermaid = false;
    let mut save_target: Option<PathBuf> = None;
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if matches!(arg.as_str(), "-h" | "--help") {
            return Ok(CliCommand::Help);
        }

        if matches!(arg.as_str(), "-v" | "--version") {
            return Ok(CliCommand::Version);
        }

        if matches!(arg.as_str(), "-g" | "--graph-only") {
            if mode != OutputMode::Default {
                return Err(anyhow!("conflicting graph options supplied"));
            }
            mode = OutputMode::GraphOnly;
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-G" | "--graph-summary") {
            // This flag is now just an alias for the default behavior (backwards compatibility)
            if mode != OutputMode::Default {
                return Err(anyhow!("conflicting graph options supplied"));
            }
            // mode stays as Default - this is now a no-op for backwards compatibility
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-dd" | "--data-downloads") {
            include_data_downloads = true;
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-m" | "--mermaid") {
            include_mermaid = true;
            i += 1;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--save=") {
            if save_target.is_some() {
                return Err(anyhow!("--save specified multiple times"));
            }
            let path = if value.is_empty() {
                PathBuf::from(".")
            } else {
                PathBuf::from(value)
            };
            save_target = Some(path);
            i += 1;
            continue;
        }

        if let Some(value) = arg.strip_prefix("-s=") {
            if save_target.is_some() {
                return Err(anyhow!("--save specified multiple times"));
            }
            let path = if value.is_empty() {
                PathBuf::from(".")
            } else {
                PathBuf::from(value)
            };
            save_target = Some(path);
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-s" | "--save") {
            if save_target.is_some() {
                return Err(anyhow!("--save specified multiple times"));
            }
            let next_is_path = url.is_some()
                && args
                    .get(i + 1)
                    .map(|next| !next.starts_with('-'))
                    .unwrap_or(false);

            if next_is_path {
                save_target = Some(PathBuf::from(args[i + 1].clone()));
                i += 2;
            } else {
                save_target = Some(PathBuf::from("."));
                i += 1;
            }

            continue;
        }

        if arg.starts_with('-') {
            return Err(anyhow!("unknown flag: {arg}"));
        }

        if url.is_none() {
            url = Some(arg.clone());
        } else {
            return Err(anyhow!("unexpected additional argument: {}", arg));
        }

        i += 1;
    }

    let url = url.ok_or_else(|| anyhow!("missing <url> or <json-ld> argument"))?;

    // Detect if input is JSON-LD or a URL
    let input = if url.trim().starts_with('{') || url.trim().starts_with('[') {
        InputSource::JsonLd(url)
    } else {
        InputSource::Url(url)
    };

    Ok(CliCommand::Run(CliOptions {
        input,
        mode,
        include_data_downloads,
        include_mermaid,
        save_target,
    }))
}

fn print_help() {
    println!("{APP_NAME} — A semantic lens for the web");
    println!("Usage: {APP_NAME} [OPTIONS] <URL|JSON-LD>\n");
    println!("Arguments:");
    println!("  <URL>         A web page URL to fetch and extract JSON-LD from");
    println!("  <JSON-LD>     Direct JSON-LD input (must start with '{{' or '[')\n");
    println!("Options:");
    println!("  -g, --graph-only        Output condensed graph summary only (no markdown)");
    println!("  -G, --graph-summary     Include product summaries and condensed graph (alias for default)");
    println!("  -m, --mermaid           Include Mermaid diagram visualization of the knowledge graph");
    println!("  -dd, --data-downloads   Include DataDownload references in output");
    println!("  -s, --save [PATH]       Save markdown output to file");
    println!("  -v, --version           Show version information");
    println!("  -h, --help              Show this help message\n");
    println!("Default behavior (no flags): Shows product summaries + markdown");
}

fn print_version() {
    println!("{APP_NAME} {VERSION}");
}

#[tokio::main]
async fn main() -> Result<()> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    match parse_arguments(&raw_args)? {
        CliCommand::Help => {
            print_help();
            return Ok(());
        }
        CliCommand::Version => {
            print_version();
            return Ok(());
        }
        CliCommand::Run(options) => run(options).await,
    }
}

async fn run(options: CliOptions) -> Result<()> {
    let (base_url, markdown, json_ld_blocks) = match &options.input {
        InputSource::Url(url) => {
            let parsed_url = Url::parse(url).context("invalid URL")?;
            let html = parser::fetch_html(parsed_url.as_str()).await?;
            let markdown = parser::html_to_markdown(&html);
            let json_ld_blocks = parser::extract_json_ld_blocks(&html)?;
            (url.clone(), markdown, json_ld_blocks)
        }
        InputSource::JsonLd(json_ld) => {
            // For direct JSON-LD input, use a placeholder URL and no markdown
            let base_url = "https://example.com/".to_string();
            let markdown = String::new();
            let json_ld_blocks = vec![json_ld.clone()];
            (base_url, markdown, json_ld_blocks)
        }
    };

    let mut loader = ReqwestLoader::default();
    let mut builder = GraphBuilder::new();

    // Combine multiple JSON-LD blocks into a single graph
    let combined_doc = parser::combine_json_ld_blocks(&json_ld_blocks)?;
    if let Ok(expanded) = ld_graph::expand_json_ld(&base_url, &combined_doc, &mut loader).await {
        builder.ingest_document(&expanded);
    }

    let graph = builder.into_graph();
    let graph_json_value = serde_json::to_value(&graph)?;
    let mut insights = GraphInsights::from(&graph);

    let include_data_downloads = options.include_data_downloads || matches!(options.mode, OutputMode::Default);

    if include_data_downloads {
        insights.data_downloads = render_data_downloads(&graph_json_value);
        if !insights.data_downloads.is_empty() {
            insights.graph_summary.push(format!(
                "ProductGroup → DataDownload ({})",
                insights.data_downloads.len()
            ));
        }
    }

    // Determine what to include based on mode
    let include_markdown = matches!(options.mode, OutputMode::Default);
    let include_summary_sections = matches!(options.mode, OutputMode::Default);
    let include_condensed_summary = matches!(options.mode, OutputMode::GraphOnly);
    let include_graph_exports = options.include_mermaid;

    let graph_json_string = if include_graph_exports {
        Some(serde_json::to_string_pretty(&graph_json_value)?)
    } else {
        None
    };

    let mermaid_diagram = if include_graph_exports {
        Some(graph_to_mermaid(&graph))
    } else {
        None
    };

    let mut output = String::new();

    // 1. Markdown first (if enabled)
    if include_markdown {
        push_section_header(&mut output, "📝", "Source Page (Markdown)");
        output.push_str(markdown.trim());
        output.push('\n');
    }

    // 2. Product summaries and structured data
    if include_summary_sections {
        // Organization first (only if exists)
        for org in &insights.organizations {
            render_organization(&mut output, org);
        }

        // ContactPoint (from other_entities, only if exists)
        for entity in &insights.other_entities {
            if entity.entity_type == "ContactPoint" {
                render_entity(&mut output, entity);
            }
        }

        // Breadcrumbs (only if exists)
        for breadcrumb in &insights.breadcrumbs {
            if !breadcrumb.items.is_empty() {
                render_breadcrumb(&mut output, breadcrumb);
            }
        }

        // Product/ProductGroup
        for pg in &insights.product_groups {
            let title = pg
                .name
                .clone()
                .unwrap_or_else(|| "ProductGroup".to_string());
            push_section_header(&mut output, "📦", &format!("ProductGroup: {title}"));
            if let Some(id) = pg.product_group_id.as_ref() {
                push_key_value(&mut output, "ProductGroup ID", id);
            }
            if let Some(brand) = pg.brand.as_ref() {
                push_key_value(&mut output, "Brand", brand);
            }
            if !pg.varies_by.is_empty() {
                push_key_value(&mut output, "Varies By", &pg.varies_by.join(", "));
            }
            if pg.total_variants > 0 {
                push_key_value(
                    &mut output,
                    "Total Variants",
                    &pg.total_variants.to_string(),
                );
            }
            if let Some(stats) = pg.price_stats.as_ref() {
                push_key_value(&mut output, "Price Range", &format_price_range(stats));
            }
            if let Some(avail) = format_availability_counts(&pg.availability_counts) {
                push_key_value(&mut output, "Availability", &avail);
            }
            
            // Display common properties (non-varies properties shared by all variants)
            if !pg.common_properties.is_empty() {
                let _ = writeln!(&mut output);
                let _ = writeln!(&mut output, "**Common Properties** (shared by all variants):");
                
                // Sort keys for consistent display
                let mut sorted_keys: Vec<_> = pg.common_properties.keys().collect();
                sorted_keys.sort();
                
                for key in sorted_keys {
                    if let Some(value) = pg.common_properties.get(key) {
                        // Skip name as it's already shown in the title
                        if key.to_lowercase() != "name" {
                            push_key_value(&mut output, key, value);
                        }
                    }
                }
            }
            
            let _ = writeln!(&mut output);

            // Render variants for this product group
            if !pg.variants.is_empty() {
                render_variant_table(&mut output, &pg.variants, &pg.varies_by, pg.total_variants);
            }
        }

        // 5. Web pages
        for webpage in &insights.web_pages {
            render_webpage(&mut output, webpage);
        }

        // 6. Other entities (except ContactPoint which was already rendered)
        for entity in &insights.other_entities {
            if entity.entity_type != "ContactPoint" {
                render_entity(&mut output, entity);
            }
        }
    }

    if include_data_downloads {
        render_data_downloads_section(&mut output, &insights.data_downloads);
    }

    if include_condensed_summary {
        render_graph_summary(&mut output, &insights.graph_summary);
    }

    if let Some(json_pretty) = graph_json_string.as_ref() {
        push_section_header(&mut output, "🧾", "Knowledge Graph JSON");
        output.push_str("```json\n");
        output.push_str(json_pretty);
        output.push_str("\n```\n");
    }

    if let Some(mermaid) = mermaid_diagram.as_ref() {
        push_section_header(&mut output, "🕸️", "Knowledge Graph Visualization");
        output.push_str("```mermaid\n");
        output.push_str(mermaid);
        output.push_str("\n```\n");
    }

    print!("{}", output);

    if let Some(save_base) = options.save_target {
        let parsed_url = Url::parse(&base_url)?;
        let output_path = build_output_path(&save_base, &parsed_url);
        if let Some(parent) = output_path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create output directory {}", parent.display())
            })?;
        }

        fs::write(&output_path, output.as_bytes())
            .with_context(|| format!("failed to write output file {}", output_path.display()))?;

        println!("\nWrote output to {}", output_path.display());
    }

    Ok(())
}

enum NodeLabel {
    Plain(String),
    Link { href: String, text: String },
}

const DIVIDER: &str = "─────────────────────────────────────────────────────────────";
const LABEL_WIDTH: usize = 16;

fn push_section_header(buf: &mut String, icon: &str, title: &str) {
    let _ = writeln!(buf, "{DIVIDER}");
    let _ = writeln!(buf, "{icon} {title}");
    let _ = writeln!(buf, "{DIVIDER}");
}

fn push_key_value(buf: &mut String, label: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    let _ = writeln!(buf, "• {:<width$} : {}", label, value, width = LABEL_WIDTH);
}

#[derive(Default)]
struct GraphInsights {
    product_groups: Vec<ProductGroupSummary>,
    organizations: Vec<OrganizationSummary>,
    web_pages: Vec<WebPageSummary>,
    breadcrumbs: Vec<BreadcrumbSummary>,
    other_entities: Vec<EntitySummary>,
    graph_summary: Vec<String>,
    data_downloads: Vec<DataDownloadEntry>,
}

#[derive(Default)]
struct OrganizationSummary {
    name: Option<String>,
    logo: Option<String>,
    telephone: Option<String>,
    email: Option<String>,
    address: Option<AddressSummary>,
    rating: Option<RatingSummary>,
}

#[derive(Default)]
struct AddressSummary {
    street: Option<String>,
    locality: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

#[derive(Default)]
struct RatingSummary {
    rating_value: Option<String>,
    review_count: Option<String>,
}

#[derive(Default)]
struct WebPageSummary {
    url: Option<String>,
    speakable: Option<String>,
}

#[derive(Default)]
struct EntitySummary {
    id: String,
    entity_type: String,
    properties: HashMap<String, String>,
}

#[derive(Default)]
struct BreadcrumbSummary {
    items: Vec<BreadcrumbItem>,
}

#[derive(Default, Clone)]
struct BreadcrumbItem {
    position: usize,
    name: String,
    url: String,
}

#[derive(Default)]
struct ProductGroupSummary {
    name: Option<String>,
    product_group_id: Option<String>,
    brand: Option<String>,
    varies_by: Vec<String>,
    common_properties: HashMap<String, String>, // Properties shared by all variants (not in variesBy)
    variants: Vec<VariantSummary>,
    total_variants: usize,
    price_stats: Option<PriceStats>,
    availability_counts: HashMap<String, usize>,
}

#[derive(Default)]
struct PriceStats {
    min: f64,
    max: f64,
    currency: Option<String>,
}

#[derive(Default)]
struct VariantSummary {
    sku: Option<String>,
    color: Option<String>,
    size: Option<String>,
    frame_shape: Option<String>,
    battery: Option<String>,
    price_display: Option<String>,
    price_numeric: Option<f64>,
    price_currency: Option<String>,
    availability: Option<String>,
    additional: HashMap<String, String>,
}

struct DataDownloadEntry {
    content_url: String,
    encoding_format: Option<String>,
    license: Option<String>,
}

type OfferInfo = (Option<String>, Option<f64>, Option<String>, Option<String>);

fn predicate_matches(predicate: &str, name: &str) -> bool {
    predicate.ends_with(name)
}

fn build_adjacency<'a>(graph: &'a KnowledgeGraph) -> HashMap<&'a str, Vec<&'a GraphEdge>> {
    let mut map: HashMap<&'a str, Vec<&'a GraphEdge>> = HashMap::new();
    for edge in &graph.edges {
        map.entry(edge.from.as_str()).or_default().push(edge);
    }
    map
}

fn property_list(node: &GraphNode, keys: &[&str]) -> Vec<String> {
    for key in keys {
        if let Some(value) = resolve_node_property(&node.properties, key) {
            return match value {
                JsonValue::Array(items) => items.iter().map(json_value_display).collect(),
                _ => vec![json_value_display(value)],
            };
        }
    }
    Vec::new()
}

impl GraphInsights {
    fn from(graph: &KnowledgeGraph) -> Self {
        let nodes_map: HashMap<&str, &GraphNode> = graph
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();
        let adjacency = build_adjacency(graph);

        let mut insights = GraphInsights::default();
        let mut property_names = BTreeSet::new();
        let mut direct_properties = BTreeSet::new();
        let mut offer_count = 0usize;

        // Process ALL ProductGroups (not just the first one)
        for product_group in nodes_map.values().filter(|node| has_schema_type(node, "ProductGroup")) {
            let mut pg_summary = ProductGroupSummary {
                name: property_text(product_group, &["https://schema.org/name", "name"]),
                product_group_id: property_text(
                    product_group,
                    &[
                        "https://schema.org/productGroupID",
                        "http://schema.org/productGroupID",
                        "productGroupID",
                    ],
                ),
                varies_by: property_list(
                    product_group,
                    &[
                        "https://schema.org/variesBy",
                        "http://schema.org/variesBy",
                        "variesBy",
                    ],
                ),
                ..Default::default()
            };

            let mut brand_added = false;

            if let Some(edges) = adjacency.get(product_group.id.as_str()) {
                for edge in edges {
                    let target_id = edge.to.as_str();
                    if predicate_matches(&edge.predicate, "brand") {
                        if let Some(brand_node) = nodes_map.get(target_id) {
                            pg_summary.brand =
                                property_text(brand_node, &["https://schema.org/name", "name"]);
                            if !brand_added {
                                if let Some(brand_name) = pg_summary.brand.as_ref() {
                                    insights
                                        .graph_summary
                                        .push(format!("ProductGroup → Brand ({brand_name})"));
                                } else {
                                    insights
                                        .graph_summary
                                        .push("ProductGroup → Brand".to_string());
                                }
                                brand_added = true;
                            }
                        }
                    } else if predicate_matches(&edge.predicate, "hasVariant")
                        && let Some(product_node) = nodes_map.get(target_id)
                    {
                        let variant = summarize_variant(
                            product_node,
                            &adjacency,
                            &nodes_map,
                            &mut property_names,
                            &mut direct_properties,
                            &mut offer_count,
                        );
                        if let Some(status) = variant.availability.as_ref() {
                            *pg_summary
                                .availability_counts
                                .entry(status.clone())
                                .or_insert(0) += 1;
                        }
                        if let Some(price) = variant.price_numeric {
                            pg_summary.price_stats = match pg_summary.price_stats.take() {
                                Some(mut stats) => {
                                    if price < stats.min {
                                        stats.min = price;
                                    }
                                    if price > stats.max {
                                        stats.max = price;
                                    }
                                    if stats.currency.is_none() {
                                        stats.currency = variant.price_currency.clone();
                                    }
                                    Some(stats)
                                }
                                None => Some(PriceStats {
                                    min: price,
                                    max: price,
                                    currency: variant.price_currency.clone(),
                                }),
                            };
                        }
                        pg_summary.total_variants += 1;
                        pg_summary.variants.push(variant);
                    }
                }
            }

            if pg_summary.total_variants > 0 {
                insights.graph_summary.push(format!(
                    "ProductGroup → Product ({})",
                    pg_summary.total_variants
                ));
            }

            // Sort variants within this product group
            pg_summary.variants.sort_by(|a, b| a.sku.cmp(&b.sku));
            
            // Extract common properties (properties not in variesBy) from the first variant
            if let Some(first_variant) = pg_summary.variants.first() {
                // Get the first variant's product node to extract direct properties
                if let Some(edges) = adjacency.get(product_group.id.as_str()) {
                    for edge in edges {
                        if predicate_matches(&edge.predicate, "hasVariant") {
                            if let Some(variant_node) = nodes_map.get(edge.to.as_str()) {
                                // Check if this is the first variant by SKU
                                let variant_sku = property_text(variant_node, &["https://schema.org/sku", "http://schema.org/sku", "sku"]);
                                if variant_sku == first_variant.sku {
                                    pg_summary.common_properties = extract_common_properties(
                                        variant_node,
                                        &adjacency,
                                        &nodes_map,
                                        &pg_summary.varies_by,
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            
            insights.product_groups.push(pg_summary);
        }

        // If no ProductGroups found, check for standalone Products
        if insights.product_groups.is_empty()
            && let Some(product_node) = nodes_map
                .values()
                .find(|node| has_schema_type(node, "Product"))
        {
            let mut pg_summary = ProductGroupSummary {
                name: property_text(
                    product_node,
                    &["https://schema.org/name", "http://schema.org/name", "name"],
                ),
                product_group_id: property_text(
                    product_node,
                    &[
                        "https://schema.org/productID",
                        "http://schema.org/productID",
                        "productID",
                        "https://schema.org/sku",
                        "http://schema.org/sku",
                        "sku",
                    ],
                ),
                ..Default::default()
            };

            if let Some(edges) = adjacency.get(product_node.id.as_str()) {
                for edge in edges {
                    let target_id = edge.to.as_str();
                    if predicate_matches(&edge.predicate, "brand")
                        && let Some(brand_node) = nodes_map.get(target_id)
                    {
                        pg_summary.brand = property_text(
                            brand_node,
                            &["https://schema.org/name", "http://schema.org/name", "name"],
                        );
                        if let Some(brand_name) = pg_summary.brand.as_ref() {
                            insights
                                .graph_summary
                                .push(format!("Product → Brand ({brand_name})"));
                        } else {
                            insights.graph_summary.push("Product → Brand".to_string());
                        }
                    }
                }
            }

            let variant = summarize_variant(
                product_node,
                &adjacency,
                &nodes_map,
                &mut property_names,
                &mut direct_properties,
                &mut offer_count,
            );

            if let Some(status) = variant.availability.as_ref() {
                *pg_summary
                    .availability_counts
                    .entry(status.clone())
                    .or_insert(0) += 1;
            }

            if let Some(price) = variant.price_numeric {
                pg_summary.price_stats = Some(PriceStats {
                    min: price,
                    max: price,
                    currency: variant.price_currency.clone(),
                });
            }

            pg_summary.total_variants = 1;
            pg_summary.variants.push(variant);
            insights.product_groups.push(pg_summary);
        }

        if offer_count > 0 {
            insights
                .graph_summary
                .push(format!("Product → Offer ({offer_count})"));
        }

        if !property_names.is_empty() {
            let joined = property_names.into_iter().collect::<Vec<_>>().join(", ");
            insights
                .graph_summary
                .push(format!("Product → PropertyValue ({joined})"));
        }

        if !direct_properties.is_empty() {
            for prop in direct_properties {
                insights
                    .graph_summary
                    .push(format!("Product → {}", title_case(&prop)));
            }
        }

        // Extract Organization entities
        for node in nodes_map.values() {
            if has_schema_type(node, "Organization") {
                insights.organizations.push(extract_organization(node, &adjacency, &nodes_map));
                insights.graph_summary.push("Organization".to_string());
            }
        }

        // Extract WebPage entities
        for node in nodes_map.values() {
            if has_schema_type(node, "WebPage") {
                insights.web_pages.push(extract_webpage(node));
                insights.graph_summary.push("WebPage".to_string());
            }
        }

        // Extract BreadcrumbList
        for node in nodes_map.values() {
            if has_schema_type(node, "BreadcrumbList") {
                insights.breadcrumbs.push(extract_breadcrumb(node, &adjacency, &nodes_map));
                insights.graph_summary.push("BreadcrumbList".to_string());
            }
        }

        // Extract other interesting entities
        for node in nodes_map.values() {
            // Skip entities we already process elsewhere or are sub-entities
            if has_schema_type(node, "Product")
                || has_schema_type(node, "ProductGroup")
                || has_schema_type(node, "Organization")
                || has_schema_type(node, "WebPage")
                || has_schema_type(node, "Offer")
                || has_schema_type(node, "Brand")
                || has_schema_type(node, "PropertyValue")
                || has_schema_type(node, "PostalAddress")
                || has_schema_type(node, "AggregateRating")
                || has_schema_type(node, "SpeakableSpecification")
                || has_schema_type(node, "BreadcrumbList")
                || has_schema_type(node, "ListItem")
            {
                continue;
            }

            // Collect interesting types
            for node_type in &node.types {
                let short_type = shorten_iri(node_type);
                if !short_type.starts_with('_') && short_type != "@graph" {
                    insights.other_entities.push(extract_generic_entity(node));
                    insights.graph_summary.push(short_type);
                    break; // Only add once per node
                }
            }
        }

        insights
    }
}

fn summarize_variant<'a>(
    product: &GraphNode,
    adjacency: &HashMap<&'a str, Vec<&'a GraphEdge>>,
    nodes: &HashMap<&'a str, &'a GraphNode>,
    property_names: &mut BTreeSet<String>,
    direct_properties: &mut BTreeSet<String>,
    offer_count: &mut usize,
) -> VariantSummary {
    let mut summary = VariantSummary {
        sku: property_text(
            product,
            &["https://schema.org/sku", "http://schema.org/sku", "sku"],
        ),
        color: property_text(
            product,
            &[
                "https://schema.org/color",
                "http://schema.org/color",
                "color",
            ],
        ),
        size: property_text(
            product,
            &["https://schema.org/size", "http://schema.org/size", "size"],
        ),
        ..Default::default()
    };

    if summary.color.is_some() {
        direct_properties.insert("color".to_string());
    }
    if summary.size.is_some() {
        direct_properties.insert("size".to_string());
    }

    let mut additional = collect_additional_properties(product, adjacency, nodes);
    
    // Check for isVariantOf to inherit properties from parent variant
    if let Some(edges) = adjacency.get(product.id.as_str()) {
        for edge in edges {
            if predicate_matches(&edge.predicate, "isVariantOf") {
                if let Some(parent_node) = nodes.get(edge.to.as_str()) {
                    let parent_additional = collect_additional_properties(parent_node, adjacency, nodes);
                    // Inherit properties that aren't already set
                    for (key, value) in parent_additional {
                        additional.entry(key).or_insert(value);
                    }
                }
            }
        }
    }
    
    // Support both FrameShape and FrameType
    summary.frame_shape = additional.get("FrameShape").cloned()
        .or_else(|| additional.get("FrameType").cloned());
    summary.battery = additional.get("BatteryCapacity").cloned();
    for key in additional.keys() {
        property_names.insert(key.clone());
    }
    summary.additional = additional;

    if let Some((price_display, price_numeric, price_currency, availability)) =
        extract_offer(product, adjacency, nodes)
    {
        summary.price_display = price_display;
        summary.price_numeric = price_numeric;
        summary.price_currency = price_currency.clone();
        summary.availability = availability.clone();
        *offer_count += 1;
    }

    summary
}

fn collect_additional_properties<'a>(
    product: &GraphNode,
    adjacency: &HashMap<&'a str, Vec<&'a GraphEdge>>,
    nodes: &HashMap<&'a str, &'a GraphNode>,
) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let Some(edges) = adjacency.get(product.id.as_str()) {
        for edge in edges {
            if predicate_matches(&edge.predicate, "additionalProperty")
                && let Some(node) = nodes.get(edge.to.as_str())
                && has_schema_type(node, "PropertyValue")
                && let Some(name) = property_text(
                    node,
                    &["https://schema.org/name", "http://schema.org/name", "name"],
                )
                && let Some(value) = property_text(
                    node,
                    &[
                        "https://schema.org/value",
                        "http://schema.org/value",
                        "value",
                    ],
                )
            {
                result.insert(name.clone(), value);
            }
        }
    }
    result
}

fn extract_common_properties<'a>(
    product: &GraphNode,
    adjacency: &HashMap<&'a str, Vec<&'a GraphEdge>>,
    nodes: &HashMap<&'a str, &'a GraphNode>,
    varies_by: &[String],
) -> HashMap<String, String> {
    let mut common = HashMap::new();
    
    // Normalize variesBy to lowercase for case-insensitive comparison
    let varies_by_lower: Vec<String> = varies_by.iter()
        .map(|s| s.to_lowercase())
        .collect();
    
    // Helper function to normalize property names by extracting tokens
    // e.g., "FrameSize" -> ["frame", "size"], "BatteryCapacity" -> ["battery", "capacity"]
    let normalize_tokens = |s: &str| -> Vec<String> {
        // Split on case boundaries and non-alphanumeric characters
        let mut tokens = Vec::new();
        let mut current = String::new();
        
        for ch in s.chars() {
            if ch.is_uppercase() && !current.is_empty() {
                tokens.push(current.to_lowercase());
                current = String::new();
            }
            if ch.is_alphanumeric() {
                current.push(ch);
            } else if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current = String::new();
            }
        }
        if !current.is_empty() {
            tokens.push(current.to_lowercase());
        }
        tokens
    };
    
    // Helper function to check if a property varies
    let is_varying = |prop_name: &str| -> bool {
        let prop_lower = prop_name.to_lowercase();
        let prop_tokens = normalize_tokens(prop_name);
        
        varies_by_lower.iter().any(|vb| {
            // Exact match
            if vb == &prop_lower {
                return true;
            }
            
            // Token-based matching: if all tokens from variesBy appear in the property name
            // e.g., "size" in variesBy matches "FrameSize" (tokens: ["frame", "size"])
            // but not "colorway" (tokens: ["colorway"])
            let vb_tokens = normalize_tokens(vb);
            
            // Check if all variesBy tokens are present in property tokens
            // This handles cases like "Size" matching "FrameSize"
            // but avoids false positives like "Color" matching "Colorway"
            if vb_tokens.len() == 1 && prop_tokens.contains(&vb_tokens[0]) {
                return true;
            }
            
            // For multi-token variesBy (e.g., "FrameSize"), require exact token sequence
            if vb_tokens.len() > 1 && vb_tokens == prop_tokens {
                return true;
            }
            
            false
        })
    };
    
    // Extract direct properties from the product node
    let direct_props = vec![
        ("name", &["https://schema.org/name", "http://schema.org/name", "name"] as &[&str]),
        ("description", &["https://schema.org/description", "http://schema.org/description", "description"]),
        ("material", &["https://schema.org/material", "http://schema.org/material", "material"]),
        ("color", &["https://schema.org/color", "http://schema.org/color", "color"]),
        ("size", &["https://schema.org/size", "http://schema.org/size", "size"]),
        ("brand", &["https://schema.org/brand", "http://schema.org/brand", "brand"]),
        ("model", &["https://schema.org/model", "http://schema.org/model", "model"]),
        ("category", &["https://schema.org/category", "http://schema.org/category", "category"]),
        ("width", &["https://schema.org/width", "http://schema.org/width", "width"]),
        ("height", &["https://schema.org/height", "http://schema.org/height", "height"]),
        ("depth", &["https://schema.org/depth", "http://schema.org/depth", "depth"]),
        ("weight", &["https://schema.org/weight", "http://schema.org/weight", "weight"]),
    ];
    
    for (prop_name, prop_paths) in direct_props {
        // Skip if this property varies
        if is_varying(prop_name) {
            continue;
        }
        
        if let Some(value) = property_text(product, prop_paths) {
            common.insert(prop_name.to_string(), value);
        }
    }
    
    // Also extract additionalProperty items that aren't in variesBy
    let additional = collect_additional_properties(product, adjacency, nodes);
    for (key, value) in additional {
        if !is_varying(&key) {
            common.insert(key, value);
        }
    }
    
    common
}

fn extract_offer<'a>(
    product: &GraphNode,
    adjacency: &HashMap<&'a str, Vec<&'a GraphEdge>>,
    nodes: &HashMap<&'a str, &'a GraphNode>,
) -> Option<OfferInfo> {
    let edges = adjacency.get(product.id.as_str())?;
    for edge in edges {
        if predicate_matches(&edge.predicate, "offers")
            && let Some(offer_node) = nodes.get(edge.to.as_str())
        {
            if !has_schema_type(offer_node, "Offer") {
                continue;
            }
            let price_raw = property_text(
                offer_node,
                &[
                    "https://schema.org/price",
                    "http://schema.org/price",
                    "price",
                ],
            );
            let currency = property_text(
                offer_node,
                &[
                    "https://schema.org/priceCurrency",
                    "http://schema.org/priceCurrency",
                    "priceCurrency",
                ],
            );
            let price_numeric = price_raw
                .as_ref()
                .and_then(|raw| raw.replace(',', ".").parse::<f64>().ok());
            let price_display = if let Some(value) = price_numeric {
                Some(format_price(value, currency.as_deref()))
            } else {
                price_raw.clone()
            };

            let availability = property_text(
                offer_node,
                &[
                    "https://schema.org/availability",
                    "http://schema.org/availability",
                    "availability",
                ],
            )
            .map(|s| shorten_iri(&s));

            return Some((price_display, price_numeric, currency, availability));
        }
    }
    None
}

fn extract_organization<'a>(
    node: &GraphNode,
    adjacency: &HashMap<&'a str, Vec<&'a GraphEdge>>,
    nodes_map: &HashMap<&'a str, &'a GraphNode>,
) -> OrganizationSummary {
    let mut org = OrganizationSummary {
        name: property_text(node, &["https://schema.org/name", "http://schema.org/name", "name"]),
        logo: property_text(node, &["https://schema.org/logo", "http://schema.org/logo", "logo"]),
        telephone: property_text(node, &["https://schema.org/telephone", "http://schema.org/telephone", "telephone"]),
        email: property_text(node, &["https://schema.org/email", "http://schema.org/email", "email"]),
        ..Default::default()
    };

    // Extract address if present
    if let Some(edges) = adjacency.get(node.id.as_str()) {
        for edge in edges {
            if predicate_matches(&edge.predicate, "address") {
                if let Some(address_node) = nodes_map.get(edge.to.as_str()) {
                    org.address = Some(AddressSummary {
                        street: property_text(address_node, &["https://schema.org/streetAddress", "http://schema.org/streetAddress", "streetAddress"]),
                        locality: property_text(address_node, &["https://schema.org/addressLocality", "http://schema.org/addressLocality", "addressLocality"]),
                        postal_code: property_text(address_node, &["https://schema.org/postalCode", "http://schema.org/postalCode", "postalCode"]),
                        country: property_text(address_node, &["https://schema.org/addressCountry", "http://schema.org/addressCountry", "addressCountry"]),
                    });
                }
            } else if predicate_matches(&edge.predicate, "aggregateRating") {
                if let Some(rating_node) = nodes_map.get(edge.to.as_str()) {
                    org.rating = Some(RatingSummary {
                        rating_value: property_text(rating_node, &["https://schema.org/ratingValue", "http://schema.org/ratingValue", "ratingValue"]),
                        review_count: property_text(rating_node, &["https://schema.org/reviewCount", "http://schema.org/reviewCount", "reviewCount"]),
                    });
                }
            }
        }
    }

    org
}

fn extract_webpage(node: &GraphNode) -> WebPageSummary {
    let speakable_text = property_text(node, &["https://schema.org/speakable", "http://schema.org/speakable", "speakable"]);
    
    WebPageSummary {
        url: property_text(node, &["https://schema.org/url", "http://schema.org/url", "url"]),
        speakable: speakable_text,
    }
}

fn extract_breadcrumb<'a>(
    node: &GraphNode,
    adjacency: &HashMap<&'a str, Vec<&'a GraphEdge>>,
    nodes_map: &HashMap<&'a str, &'a GraphNode>,
) -> BreadcrumbSummary {
    let mut items = Vec::new();

    if let Some(edges) = adjacency.get(node.id.as_str()) {
        for edge in edges {
            if predicate_matches(&edge.predicate, "itemListElement") {
                if let Some(list_item_node) = nodes_map.get(edge.to.as_str()) {
                    if has_schema_type(list_item_node, "ListItem") {
                        let position = property_text(
                            list_item_node,
                            &["https://schema.org/position", "http://schema.org/position", "position"],
                        )
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);

                        let name = property_text(
                            list_item_node,
                            &["https://schema.org/name", "http://schema.org/name", "name"],
                        )
                        .unwrap_or_default();

                        let url = property_text(
                            list_item_node,
                            &["https://schema.org/item", "http://schema.org/item", "item"],
                        )
                        .unwrap_or_default();

                        items.push(BreadcrumbItem {
                            position,
                            name,
                            url,
                        });
                    }
                }
            }
        }
    }

    BreadcrumbSummary { items }
}

fn extract_generic_entity(node: &GraphNode) -> EntitySummary {
    let mut properties = HashMap::new();
    
    // Extract all simple string properties
    for (key, value) in &node.properties {
        let short_key = shorten_iri(key);
        let value_str = json_value_display(value);
        if !value_str.is_empty() {
            properties.insert(short_key, value_str);
        }
    }

    let entity_type = node.types.first()
        .map(|t| shorten_iri(t))
        .unwrap_or_else(|| "Entity".to_string());

    EntitySummary {
        id: node.id.clone(),
        entity_type,
        properties,
    }
}

fn graph_to_mermaid(graph: &KnowledgeGraph) -> String {
    if graph.nodes.is_empty() {
        return "graph TD\n  Empty[\"No data\"]".to_string();
    }

    let mut lines = Vec::new();
    lines.push("graph TD".to_string());

    let mut id_map = HashMap::new();
    for (idx, node) in graph.nodes.iter().enumerate() {
        let mermaid_id = format!("N{idx}");
        id_map.insert(node.id.clone(), mermaid_id.clone());

        let label = node_label(node);
        let rendered_label = match &label {
            NodeLabel::Link { href, text } => format!(
                "<a href='{href}'>{text}</a>",
                href = escape_html_attr(href),
                text = escape_html_text(text)
            ),
            NodeLabel::Plain(text) => format!("\"{}\"", escape_mermaid_label(text)),
        };

        lines.push(format!(
            "  {id}[{label}]",
            id = mermaid_id,
            label = rendered_label
        ));
    }

    for edge in &graph.edges {
        if let (Some(from), Some(to)) = (id_map.get(&edge.from), id_map.get(&edge.to)) {
            lines.push(format!(
                "  {from} -->|{pred}| {to}",
                from = from,
                pred = escape_mermaid_label(&shorten_iri(&edge.predicate)),
                to = to
            ));
        }
    }

    lines.join("\n")
}

fn node_label(node: &GraphNode) -> NodeLabel {
    if let Some(summary) = property_value_summary(node) {
        return NodeLabel::Plain(summary);
    }

    let primary_label = property_text(
        node,
        &["https://schema.org/name", "http://schema.org/name", "name"],
    )
    .or_else(|| node.types.first().map(|t| shorten_iri(t)))
    .unwrap_or_else(|| shorten_iri(&node.id));

    match node.id.as_str() {
        id if id.starts_with("http://") || id.starts_with("https://") => NodeLabel::Link {
            href: id.to_string(),
            text: primary_label,
        },
        _ => NodeLabel::Plain(primary_label),
    }
}

fn property_text(node: &GraphNode, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = resolve_node_property(&node.properties, key)
            && let Some(text) = json_value_to_string(value)
        {
            return Some(text);
        }
    }
    None
}

fn resolve_node_property<'a>(
    props: &'a HashMap<String, JsonValue>,
    key: &str,
) -> Option<&'a JsonValue> {
    if let Some(value) = props.get(key) {
        return Some(value);
    }

    if key.starts_with("https://schema.org/") {
        let alt = key.replacen("https://", "http://", 1);
        if let Some(value) = props.get(&alt) {
            return Some(value);
        }
    } else if key.starts_with("http://schema.org/") {
        let alt = key.replacen("http://", "https://", 1);
        if let Some(value) = props.get(&alt) {
            return Some(value);
        }
    }

    if let Some(last) = key.rsplit('/').next()
        && let Some(value) = props.get(last)
    {
        return Some(value);
    }

    None
}

fn json_value_to_string(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Number(n) => Some(n.to_string()),
        JsonValue::Bool(b) => Some(b.to_string()),
        JsonValue::Array(arr) => arr.first().and_then(json_value_to_string),
        JsonValue::Object(obj) => obj
            .get("@value")
            .and_then(json_value_to_string)
            .or_else(|| obj.get("name").and_then(json_value_to_string)),
        _ => None,
    }
}

fn json_value_display(value: &JsonValue) -> String {
    if let Some(text) = json_value_to_string(value) {
        return text;
    }

    match value {
        JsonValue::Array(items) => {
            let parts: Vec<String> = items.iter().map(json_value_display).collect();
            parts.join(", ")
        }
        JsonValue::Object(obj) => {
            if let Some(val) = obj.get("@value") {
                return json_value_display(val);
            }
            serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
        }
        _ => value.to_string(),
    }
}

fn format_price(value: f64, currency: Option<&str>) -> String {
    if value.fract().abs() < 1e-4 {
        format_price_with_precision(value, currency, 0)
    } else {
        format_price_with_precision(value, currency, 2)
    }
}

fn currency_symbol(code: &str) -> Option<&'static str> {
    match code {
        "EUR" => Some("€"),
        "USD" => Some("$"),
        "GBP" => Some("£"),
        "JPY" => Some("¥"),
        _ => None,
    }
}

fn availability_icon(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "instock" => "✅",
        "outofstock" => "❌",
        "preorder" => "🕒",
        _ => "•",
    }
}

fn availability_label(status: &str) -> String {
    let icon = availability_icon(status);
    format!("{icon} {status}")
}

fn format_availability_counts(counts: &HashMap<String, usize>) -> Option<String> {
    if counts.is_empty() {
        return None;
    }
    let mut entries: Vec<(String, usize)> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let rendered = entries
        .into_iter()
        .map(|(status, count)| format!("{count} {status}"))
        .collect::<Vec<_>>()
        .join(" / ");
    Some(rendered)
}

fn format_price_range(stats: &PriceStats) -> String {
    if (stats.min - stats.max).abs() < 1e-4 {
        format_price_with_precision(stats.min, stats.currency.as_deref(), 2)
    } else {
        format!(
            "{} – {}",
            format_price_with_precision(stats.min, stats.currency.as_deref(), 2),
            format_price_with_precision(stats.max, stats.currency.as_deref(), 2)
        )
    }
}

fn format_price_with_precision(value: f64, currency: Option<&str>, decimals: usize) -> String {
    let number = format!("{value:.prec$}", value = value, prec = decimals);

    if let Some(code) = currency {
        if let Some(symbol) = currency_symbol(code) {
            format!("{symbol}{number}")
        } else {
            format!("{number} {code}")
        }
    } else {
        number
    }
}

fn render_variant_table(buf: &mut String, variants: &[VariantSummary], varies_by: &[String], total_variants: usize) {
    if variants.is_empty() {
        return;
    }

    push_section_header(buf, "🧩", "Variants");

    // Build dynamic headers based on variesBy
    let mut headers = vec!["SKU"];
    
    // Add columns based on variesBy
    for vary in varies_by {
        match vary.as_str() {
            "Color" => headers.push("Color"),
            "Size" | "FrameSize" => headers.push("Size"),
            "FrameType" | "FrameShape" => headers.push("FrameShape"),
            "BatteryCapacity" => headers.push("Battery"),
            _ => {}, // Skip unknown properties
        }
    }
    
    // Always add Price and Availability at the end
    headers.push("Price");
    headers.push("Availability");

    let mut rows: Vec<Vec<String>> = Vec::new();
    for variant in variants {
        let mut row = vec![variant.sku.clone().unwrap_or_else(|| "–".to_string())];
        
        // Add cells based on variesBy
        for vary in varies_by {
            let cell = match vary.as_str() {
                "Color" => variant.color.clone().unwrap_or_else(|| "–".to_string()),
                "Size" | "FrameSize" => variant.size.clone().unwrap_or_else(|| "–".to_string()),
                "FrameType" | "FrameShape" => variant
                    .frame_shape
                    .clone()
                    .or_else(|| variant.additional.get("FrameType").cloned())
                    .or_else(|| variant.additional.get("FrameShape").cloned())
                    .unwrap_or_else(|| "–".to_string()),
                "BatteryCapacity" => variant
                    .battery
                    .clone()
                    .or_else(|| variant.additional.get("BatteryCapacity").cloned())
                    .unwrap_or_else(|| "–".to_string()),
                _ => continue, // Skip unknown properties
            };
            row.push(cell);
        }
        
        // Add price and availability
        row.push(variant.price_display.clone().unwrap_or_else(|| "–".to_string()));
        row.push(
            variant
                .availability
                .as_ref()
                .map(|status| availability_label(status))
                .unwrap_or_else(|| "–".to_string())
        );
        
        rows.push(row);
    }

    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows {
        for (idx, cell) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(cell.len());
        }
    }

    let format_row = |cells: &[String]| -> String {
        let mut parts = Vec::with_capacity(cells.len());
        for (idx, cell) in cells.iter().enumerate() {
            parts.push(format!(" {:<width$} ", cell, width = widths[idx]));
        }
        format!("|{}|", parts.join("|"))
    };

    let header_cells = headers.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let _ = writeln!(buf, "{}", format_row(&header_cells));

    let mut separator_parts = Vec::new();
    for width in &widths {
        separator_parts.push(format!(" {:-<width$} ", "", width = *width));
    }
    let _ = writeln!(buf, "|{}|", separator_parts.join("|"));

    for row in rows {
        let _ = writeln!(buf, "{}", format_row(&row));
    }

    if total_variants > variants.len() {
        let remaining = total_variants - variants.len();
        let _ = writeln!(buf, "({remaining} additional variants not shown)");
    }

    let _ = writeln!(buf);
}

fn render_graph_summary(buf: &mut String, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    push_section_header(buf, "🕸️", "Graph Summary (condensed)");
    for line in lines {
        let _ = writeln!(buf, "{}", line);
    }
    let _ = writeln!(buf);
}

fn render_organization(buf: &mut String, org: &OrganizationSummary) {
    push_section_header(buf, "🏢", "Organization");
    if let Some(name) = org.name.as_ref() {
        push_key_value(buf, "Name", name);
    }
    if let Some(logo) = org.logo.as_ref() {
        push_key_value(buf, "Logo", logo);
    }
    if let Some(telephone) = org.telephone.as_ref() {
        push_key_value(buf, "Telephone", telephone);
    }
    if let Some(email) = org.email.as_ref() {
        push_key_value(buf, "Email", email);
    }
    if let Some(address) = org.address.as_ref() {
        if let Some(street) = address.street.as_ref() {
            push_key_value(buf, "Street", street);
        }
        if let Some(locality) = address.locality.as_ref() {
            push_key_value(buf, "City", locality);
        }
        if let Some(postal_code) = address.postal_code.as_ref() {
            push_key_value(buf, "Postal Code", postal_code);
        }
        if let Some(country) = address.country.as_ref() {
            push_key_value(buf, "Country", country);
        }
    }
    if let Some(rating) = org.rating.as_ref() {
        if let Some(rating_value) = rating.rating_value.as_ref() {
            push_key_value(buf, "Rating", rating_value);
        }
        if let Some(review_count) = rating.review_count.as_ref() {
            push_key_value(buf, "Review Count", review_count);
        }
    }
    let _ = writeln!(buf);
}

fn render_breadcrumb(buf: &mut String, breadcrumb: &BreadcrumbSummary) {
    if breadcrumb.items.is_empty() {
        return;
    }

    push_section_header(buf, "🍞", "Breadcrumb Navigation");
    
    let mut sorted_items = breadcrumb.items.clone();
    sorted_items.sort_by_key(|item| item.position);
    
    let breadcrumb_trail: Vec<String> = sorted_items
        .iter()
        .map(|item| format!("{} ({})", item.name, item.url))
        .collect();
    
    let _ = writeln!(buf, "{}", breadcrumb_trail.join(" → "));
    let _ = writeln!(buf);
}

fn render_webpage(buf: &mut String, webpage: &WebPageSummary) {
    // Only render if there's actual content
    if webpage.url.is_none() && webpage.speakable.is_none() {
        return;
    }
    
    push_section_header(buf, "🌐", "Web Page");
    if let Some(url) = webpage.url.as_ref() {
        push_key_value(buf, "URL", url);
    }
    if let Some(speakable) = webpage.speakable.as_ref() {
        push_key_value(buf, "Speakable", speakable);
    }
    let _ = writeln!(buf);
}

fn render_entity(buf: &mut String, entity: &EntitySummary) {
    // Only render if there are properties to show
    let has_real_id = !entity.id.starts_with("_:");
    if !has_real_id && entity.properties.is_empty() {
        return;
    }
    
    push_section_header(buf, "📋", &entity.entity_type);
    // Don't show internal blank node IDs to users
    if has_real_id {
        push_key_value(buf, "ID", &entity.id);
    }
    for (key, value) in &entity.properties {
        push_key_value(buf, key, value);
    }
    let _ = writeln!(buf);
}

fn render_data_downloads_section(buf: &mut String, entries: &[DataDownloadEntry]) {
    push_section_header(buf, "🌐", "Data Downloads");
    if entries.is_empty() {
        let _ = writeln!(buf, "No data downloads detected.");
    } else {
        let label = if entries.len() == 1 {
            "data source"
        } else {
            "data sources"
        };
        let _ = writeln!(buf, "✓ Found {} official {label}:", entries.len());
        for entry in entries {
            let _ = writeln!(buf, "  • {}", entry.content_url);
            if let Some(format) = entry.encoding_format.as_ref() {
                let _ = writeln!(buf, "    ↳ encodingFormat: {format}");
            }
            if let Some(license) = entry.license.as_ref() {
                let _ = writeln!(buf, "    ↳ license: {license}");
            }
        }
    }
    let _ = writeln!(buf);
}

fn render_data_downloads(json: &JsonValue) -> Vec<DataDownloadEntry> {
    let mut entries = Vec::new();
    let Some(nodes) = json.get("nodes").and_then(|n| n.as_array()) else {
        return entries;
    };

    for node in nodes {
        let mut is_data_download = false;
        if let Some(types) = node.get("@type") {
            match types {
                JsonValue::Array(arr) => {
                    is_data_download = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .any(|ty| ty.ends_with("DataDownload"));
                }
                JsonValue::String(s) => {
                    is_data_download = s.ends_with("DataDownload");
                }
                _ => {}
            }
        }

        if !is_data_download {
            continue;
        }

        let Some(props) = node.get("properties").and_then(|p| p.as_object()) else {
            continue;
        };

        let content_url = property_text_json(
            props,
            &[
                "https://schema.org/contentUrl",
                "http://schema.org/contentUrl",
                "contentUrl",
            ],
        );

        let Some(content_url) = content_url else {
            continue;
        };

        let encoding_format = property_text_json(
            props,
            &[
                "https://schema.org/encodingFormat",
                "http://schema.org/encodingFormat",
                "encodingFormat",
            ],
        );
        let license = property_text_json(
            props,
            &[
                "https://schema.org/license",
                "http://schema.org/license",
                "license",
            ],
        );

        entries.push(DataDownloadEntry {
            content_url,
            encoding_format,
            license,
        });
    }

    entries
}

fn property_text_json(map: &serde_json::Map<String, JsonValue>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = resolve_json_property(map, key) {
            return Some(json_value_display(value));
        }
    }
    None
}

fn resolve_json_property<'a>(
    map: &'a serde_json::Map<String, JsonValue>,
    key: &str,
) -> Option<&'a JsonValue> {
    if let Some(value) = map.get(key) {
        return Some(value);
    }

    if key.starts_with("https://schema.org/") {
        let alt = key.replacen("https://", "http://", 1);
        if let Some(value) = map.get(&alt) {
            return Some(value);
        }
    } else if key.starts_with("http://schema.org/") {
        let alt = key.replacen("http://", "https://", 1);
        if let Some(value) = map.get(&alt) {
            return Some(value);
        }
    }

    if let Some(last) = key.rsplit('/').next()
        && let Some(value) = map.get(last)
    {
        return Some(value);
    }

    None
}

fn title_case(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut result = first.to_uppercase().collect::<String>();
    result.push_str(&chars.as_str().to_lowercase());
    result
}

fn shorten_iri(iri: &str) -> String {
    iri.rsplit(&['/', '#'][..])
        .next()
        .unwrap_or(iri)
        .to_string()
}

fn has_schema_type(node: &GraphNode, schema_type: &str) -> bool {
    node.types
        .iter()
        .any(|ty| shorten_iri(ty).eq_ignore_ascii_case(schema_type))
}

fn property_value_summary(node: &GraphNode) -> Option<String> {
    if !has_schema_type(node, "PropertyValue") {
        return None;
    }

    let property = property_text(
        node,
        &[
            "https://schema.org/propertyID",
            "http://schema.org/propertyID",
            "propertyID",
            "https://schema.org/name",
            "http://schema.org/name",
            "name",
        ],
    );
    let value = property_text(
        node,
        &[
            "https://schema.org/value",
            "http://schema.org/value",
            "value",
            "https://schema.org/valueReference",
            "http://schema.org/valueReference",
            "valueReference",
        ],
    );
    let unit = property_text(
        node,
        &[
            "https://schema.org/unitText",
            "http://schema.org/unitText",
            "unitText",
            "https://schema.org/unitCode",
            "http://schema.org/unitCode",
            "unitCode",
        ],
    );

    let mut label = property.unwrap_or_else(|| "PropertyValue".to_string());
    if let Some(val) = value {
        label.push_str(": ");
        label.push_str(&val);
        if let Some(unit) = unit {
            label.push(' ');
            label.push_str(&unit);
        }
    }

    Some(label)
}

fn escape_mermaid_label(label: &str) -> String {
    label.replace('"', "\\\"")
}

fn escape_html_attr(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn escape_html_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn build_output_path(base: &Path, url: &Url) -> PathBuf {
    let has_md_extension = base
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("md"))
        .unwrap_or(false);

    if has_md_extension {
        base.to_path_buf()
    } else {
        base.join(derive_output_filename(url))
    }
}

fn derive_output_filename(url: &Url) -> String {
    let host = url.host_str().unwrap_or("page");
    let mut path_component = url.path().trim_matches('/').replace('/', "_");
    if path_component.is_empty() {
        path_component = "index".to_string();
    }

    let mut parts = Vec::new();
    parts.push(sanitize_for_filename(host));
    if !path_component.is_empty() {
        parts.push(sanitize_for_filename(&path_component));
    }

    if let Some(query) = url.query()
        && !query.is_empty()
    {
        parts.push(sanitize_for_filename(query));
    }

    format!("{}.md", parts.join("__"))
}

fn sanitize_for_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
