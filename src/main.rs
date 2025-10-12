use std::collections::{BTreeSet, HashMap, HashSet};
use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use html2md::parse_html;
use iref::IriBuf;
use json_ld::object::Literal;
use json_ld::syntax::{Parse, Value};
use json_ld::{JsonLdProcessor, RemoteDocument, ReqwestLoader};
use json_syntax::Value as SyntaxValue;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::{Map, Value as JsonValue};
use url::Url;
use uuid::Uuid;

const APP_NAME: &str = "htmlens";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, PartialEq)]
enum OutputMode {
    MarkdownOnly,
    SummaryWithMarkdown,
    GraphOnly,
}

struct CliOptions {
    url: String,
    mode: OutputMode,
    include_data_downloads: bool,
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
    let mut mode = OutputMode::MarkdownOnly;
    let mut include_data_downloads = false;
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
            if mode != OutputMode::MarkdownOnly {
                return Err(anyhow!("conflicting graph options supplied"));
            }
            mode = OutputMode::GraphOnly;
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-G" | "--graph-summary") {
            if mode != OutputMode::MarkdownOnly {
                return Err(anyhow!("conflicting graph options supplied"));
            }
            mode = OutputMode::SummaryWithMarkdown;
            i += 1;
            continue;
        }

        if matches!(arg.as_str(), "-dd" | "--data-downloads") {
            include_data_downloads = true;
            i += 1;
            continue;
        }

        if arg.starts_with("--save=") {
            if save_target.is_some() {
                return Err(anyhow!("--save specified multiple times"));
            }
            let value = &arg["--save=".len()..];
            let path = if value.is_empty() {
                PathBuf::from(".")
            } else {
                PathBuf::from(value)
            };
            save_target = Some(path);
            i += 1;
            continue;
        }

        if arg.starts_with("-s=") {
            if save_target.is_some() {
                return Err(anyhow!("--save specified multiple times"));
            }
            let value = &arg[3..];
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

    let url = url.ok_or_else(|| anyhow!("missing <url> argument"))?;

    Ok(CliCommand::Run(CliOptions {
        url,
        mode,
        include_data_downloads,
        save_target,
    }))
}

fn print_help() {
    println!("{APP_NAME} — A semantic lens for the web");
    println!("Usage: {APP_NAME} [OPTIONS] <URL>\n");
    println!("Options:");
    println!("  -g, --graph-only        Output condensed graph summary only");
    println!("  -G, --graph-summary     Include product summaries and condensed graph");
    println!("  -dd, --data-downloads   Include DataDownload references in output");
    println!("  -s, --save [PATH]       Save markdown output to file");
    println!("  -v, --version           Show version information");
    println!("  -h, --help              Show this help message");
}

fn print_version() {
    println!("{APP_NAME} {VERSION}");
}

#[tokio::main]
async fn main() -> Result<()> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    let options = parse_arguments(&raw_args)?;

    let parsed_url = Url::parse(&options.url).context("invalid URL")?;
    let html = fetch(parsed_url.as_str()).await?;
    let markdown = parse_html(&sanitize_html_for_markdown(&html));
    let json_ld_blocks = extract_json_ld_blocks(&html)?;

    let mut loader = ReqwestLoader::default();
    let mut builder = GraphBuilder::new();

    for block in json_ld_blocks {
        if let Ok(expanded) = expand_block(parsed_url.as_str(), &block, &mut loader).await {
            builder.ingest_document(&expanded);
        }
    }

    let graph = builder.into_graph();
    let graph_json_value = serde_json::to_value(&graph)?;
    let mut insights = GraphInsights::from(&graph);

    if options.include_data_downloads {
        insights.data_downloads = render_data_downloads(&graph_json_value);
        if !insights.data_downloads.is_empty() {
            insights.graph_summary.push(format!(
                "ProductGroup → DataDownload ({})",
                insights.data_downloads.len()
            ));
        }
    }

    let include_markdown = !options.graph_only;
    let include_graph_sections = if options.graph_only {
        false
    } else {
        options.include_graph_sections
    };

    let graph_json_string = if include_graph_sections {
        Some(serde_json::to_string_pretty(&graph_json_value)?)
    } else {
        None
    };

    let mermaid_diagram = if include_graph_sections {
        Some(graph_to_mermaid(&graph))
    } else {
        None
    };

    let mut output = String::new();

    if let Some(pg) = insights.product_group.as_ref() {
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
        let _ = writeln!(&mut output);
    }

    if !insights.variants.is_empty() {
        let total_variants = insights
            .product_group
            .as_ref()
            .map(|pg| pg.total_variants)
            .unwrap_or_else(|| insights.variants.len());
        render_variant_table(&mut output, &insights.variants, total_variants);
    }

    if options.include_data_downloads {
        render_data_downloads_section(&mut output, &insights.data_downloads);
    }

    render_graph_summary(&mut output, &insights.graph_summary);

    if include_markdown {
        push_section_header(&mut output, "📝", "Source Page (Markdown)");
        output.push_str(markdown.trim());
        output.push('\n');
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

async fn fetch(url: &str) -> Result<String> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to fetch {}", url))?;

    response
        .error_for_status()
        .with_context(|| format!("non-success status from {}", url))?
        .text()
        .await
        .with_context(|| format!("failed to read response body from {}", url))
}

fn extract_json_ld_blocks(html: &str) -> Result<Vec<String>> {
    let document = Html::parse_document(html);
    let script_selector =
        Selector::parse("script").map_err(|e| anyhow!("unable to parse selector: {e}"))?;

    Ok(document
        .select(&script_selector)
        .filter_map(|element| {
            let script_type = element
                .value()
                .attr("type")
                .map(|t| t.trim().to_ascii_lowercase())
                .unwrap_or_default();

            if script_type.contains("ld+json") {
                let text = element.text().collect::<String>().trim().to_string();
                if text.is_empty() { None } else { Some(text) }
            } else {
                None
            }
        })
        .collect())
}

async fn expand_block(
    base_url: &str,
    raw_json_ld: &str,
    loader: &mut ReqwestLoader,
) -> Result<json_ld::ExpandedDocument> {
    let (value, _) = Value::parse_str(raw_json_ld)
        .map_err(|err| anyhow!("failed to parse JSON-LD block: {err}"))?;

    let base_iri = IriBuf::new(base_url.to_string())
        .map_err(|_| anyhow!("invalid URL provided: {base_url}"))?;

    let remote = RemoteDocument::new(
        Some(base_iri),
        Some("application/ld+json".parse().unwrap()),
        value,
    );

    remote
        .expand(loader)
        .await
        .map_err(|err| anyhow!("JSON-LD expansion failed: {err}"))
}

#[derive(Debug, Serialize)]
struct KnowledgeGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize, Clone)]
struct GraphNode {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    types: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    properties: HashMap<String, JsonValue>,
}

#[derive(Debug, Serialize, Clone)]
struct GraphEdge {
    from: String,
    to: String,
    predicate: String,
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
    product_group: Option<ProductGroupSummary>,
    variants: Vec<VariantSummary>,
    graph_summary: Vec<String>,
    data_downloads: Vec<DataDownloadEntry>,
}

#[derive(Default)]
struct ProductGroupSummary {
    name: Option<String>,
    product_group_id: Option<String>,
    brand: Option<String>,
    varies_by: Vec<String>,
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

impl GraphNode {
    fn new(id: String) -> Self {
        Self {
            id,
            types: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

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

        if let Some(product_group) = nodes_map
            .values()
            .find(|node| has_schema_type(node, "ProductGroup"))
        {
            let mut pg_summary = ProductGroupSummary::default();
            pg_summary.name = property_text(product_group, &["https://schema.org/name", "name"]);
            pg_summary.product_group_id = property_text(
                product_group,
                &[
                    "https://schema.org/productGroupID",
                    "http://schema.org/productGroupID",
                    "productGroupID",
                ],
            );
            pg_summary.varies_by = property_list(
                product_group,
                &[
                    "https://schema.org/variesBy",
                    "http://schema.org/variesBy",
                    "variesBy",
                ],
            );

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
                    } else if predicate_matches(&edge.predicate, "hasVariant") {
                        if let Some(product_node) = nodes_map.get(target_id) {
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
                            insights.variants.push(variant);
                        }
                    }
                }
            }

            if pg_summary.total_variants > 0 {
                insights.graph_summary.push(format!(
                    "ProductGroup → Product ({})",
                    pg_summary.total_variants
                ));
            }

            insights.product_group = Some(pg_summary);
        }

        if insights.product_group.is_none() {
            if let Some(product_node) = nodes_map
                .values()
                .find(|node| has_schema_type(node, "Product"))
            {
                let mut pg_summary = ProductGroupSummary::default();
                pg_summary.name = property_text(
                    product_node,
                    &["https://schema.org/name", "http://schema.org/name", "name"],
                );
                pg_summary.product_group_id = property_text(
                    product_node,
                    &[
                        "https://schema.org/productID",
                        "http://schema.org/productID",
                        "productID",
                        "https://schema.org/sku",
                        "http://schema.org/sku",
                        "sku",
                    ],
                );

                if let Some(edges) = adjacency.get(product_node.id.as_str()) {
                    for edge in edges {
                        let target_id = edge.to.as_str();
                        if predicate_matches(&edge.predicate, "brand") {
                            if let Some(brand_node) = nodes_map.get(target_id) {
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
                insights.variants.push(variant);
                insights.product_group = Some(pg_summary);
            }
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

        insights.variants.sort_by(|a, b| a.sku.cmp(&b.sku));

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
    let mut summary = VariantSummary::default();
    summary.sku = property_text(
        product,
        &["https://schema.org/sku", "http://schema.org/sku", "sku"],
    );
    summary.color = property_text(
        product,
        &[
            "https://schema.org/color",
            "http://schema.org/color",
            "color",
        ],
    );
    summary.size = property_text(
        product,
        &["https://schema.org/size", "http://schema.org/size", "size"],
    );
    if summary.color.is_some() {
        direct_properties.insert("color".to_string());
    }
    if summary.size.is_some() {
        direct_properties.insert("size".to_string());
    }

    let additional = collect_additional_properties(product, adjacency, nodes);
    summary.frame_shape = additional.get("FrameShape").cloned();
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
            if predicate_matches(&edge.predicate, "additionalProperty") {
                if let Some(node) = nodes.get(edge.to.as_str()) {
                    if has_schema_type(node, "PropertyValue") {
                        if let Some(name) = property_text(
                            node,
                            &["https://schema.org/name", "http://schema.org/name", "name"],
                        ) {
                            if let Some(value) = property_text(
                                node,
                                &[
                                    "https://schema.org/value",
                                    "http://schema.org/value",
                                    "value",
                                ],
                            ) {
                                result.insert(name.clone(), value);
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

fn extract_offer<'a>(
    product: &GraphNode,
    adjacency: &HashMap<&'a str, Vec<&'a GraphEdge>>,
    nodes: &HashMap<&'a str, &'a GraphNode>,
) -> Option<(Option<String>, Option<f64>, Option<String>, Option<String>)> {
    let edges = adjacency.get(product.id.as_str())?;
    for edge in edges {
        if predicate_matches(&edge.predicate, "offers") {
            if let Some(offer_node) = nodes.get(edge.to.as_str()) {
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
    }
    None
}

struct GraphBuilder {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
    processing: HashSet<String>,
}

impl GraphBuilder {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            processing: HashSet::new(),
        }
    }

    fn ingest_document(&mut self, doc: &json_ld::ExpandedDocument) {
        for object in doc.iter() {
            self.process_indexed_object(object);
        }
    }

    fn process_indexed_object(
        &mut self,
        object: &json_ld::IndexedObject<iref::IriBuf, rdf_types::BlankIdBuf>,
    ) -> Option<String> {
        match object.as_ref() {
            json_ld::Object::Node(node) => Some(self.process_node(node)),
            json_ld::Object::List(list) => {
                for item in list.iter() {
                    self.process_indexed_object(item);
                }
                None
            }
            json_ld::Object::Value(_) => None,
        }
    }

    fn process_node(
        &mut self,
        node: &json_ld::Node<iref::IriBuf, rdf_types::BlankIdBuf>,
    ) -> String {
        let node_id = node_identifier(node.id.as_ref());
        self.nodes
            .entry(node_id.clone())
            .or_insert_with(|| GraphNode::new(node_id.clone()));

        if !self.processing.insert(node_id.clone()) {
            return node_id;
        }

        if let Some(types) = &node.types {
            for ty in types {
                let ty_str = id_to_string(ty);
                let entry = self
                    .nodes
                    .get_mut(&node_id)
                    .expect("node should exist before types update");
                if !entry.types.contains(&ty_str) {
                    entry.types.push(ty_str);
                }
            }
        }

        if let Some(graph) = &node.graph {
            for object in graph.iter() {
                if let Some(target_id) = self.process_indexed_object(object) {
                    self.edges.push(GraphEdge {
                        from: node_id.clone(),
                        to: target_id,
                        predicate: "@graph".to_string(),
                    });
                }
            }
        }

        if let Some(included) = &node.included {
            for indexed_node in included.iter() {
                let target_id = self.process_node(indexed_node.as_ref());
                self.edges.push(GraphEdge {
                    from: node_id.clone(),
                    to: target_id,
                    predicate: "@included".to_string(),
                });
            }
        }

        if let Some(reverse) = &node.reverse_properties {
            for (predicate, nodes) in reverse.iter() {
                let predicate_str = id_to_string(predicate);
                for reverse_node in nodes.iter() {
                    let source_id = self.process_node(reverse_node.as_ref());
                    self.edges.push(GraphEdge {
                        from: source_id,
                        to: node_id.clone(),
                        predicate: predicate_str.clone(),
                    });
                }
            }
        }

        for (predicate, values) in node.properties.iter() {
            let predicate_str = id_to_string(predicate);
            let collected = self.collect_property_values(&node_id, &predicate_str, values);
            if !collected.is_empty() {
                for value in collected {
                    self.add_property_value(&node_id, &predicate_str, value);
                }
            }
        }

        self.processing.remove(&node_id);
        node_id
    }

    fn collect_property_values(
        &mut self,
        source_id: &str,
        predicate: &str,
        values: &[json_ld::IndexedObject<iref::IriBuf, rdf_types::BlankIdBuf>],
    ) -> Vec<JsonValue> {
        let mut collected = Vec::new();

        for value in values {
            self.handle_value(source_id, predicate, value, &mut collected);
        }

        collected
    }

    fn handle_value(
        &mut self,
        source_id: &str,
        predicate: &str,
        value: &json_ld::IndexedObject<iref::IriBuf, rdf_types::BlankIdBuf>,
        collected: &mut Vec<JsonValue>,
    ) {
        match value.as_ref() {
            json_ld::Object::Value(v) => {
                if let Some(json) = value_object_to_json(v) {
                    collected.push(json);
                }
            }
            json_ld::Object::Node(node) => {
                let target_id = self.process_node(node);
                self.edges.push(GraphEdge {
                    from: source_id.to_string(),
                    to: target_id,
                    predicate: predicate.to_string(),
                });
            }
            json_ld::Object::List(list) => {
                let mut list_values = Vec::new();
                for item in list.iter() {
                    self.handle_value(source_id, predicate, item, &mut list_values);
                }
                if !list_values.is_empty() {
                    collected.push(JsonValue::Array(list_values));
                }
            }
        }
    }

    fn add_property_value(&mut self, node_id: &str, predicate: &str, value: JsonValue) {
        let entry = self
            .nodes
            .get_mut(node_id)
            .expect("node must exist before adding properties");
        match entry.properties.get_mut(predicate) {
            Some(existing) => {
                if existing.is_array() {
                    if let Some(arr) = existing.as_array_mut() {
                        arr.push(value);
                    }
                } else {
                    let old = existing.take();
                    *existing = JsonValue::Array(vec![old, value]);
                }
            }
            None => {
                entry.properties.insert(predicate.to_string(), value);
            }
        }
    }

    fn into_graph(mut self) -> KnowledgeGraph {
        for node in self.nodes.values_mut() {
            node.types.sort();
            node.types.dedup();
        }
        let mut nodes: Vec<GraphNode> = self.nodes.into_values().collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        KnowledgeGraph {
            nodes,
            edges: self.edges,
        }
    }
}

fn value_object_to_json(value: &json_ld::Value<iref::IriBuf>) -> Option<JsonValue> {
    match value {
        json_ld::Value::Literal(lit, _) => match lit {
            Literal::Null => Some(JsonValue::Null),
            Literal::Boolean(b) => Some(JsonValue::Bool(*b)),
            Literal::Number(n) => serde_json::Number::from_str(&n.to_string())
                .ok()
                .map(JsonValue::Number),
            Literal::String(s) => Some(JsonValue::String(s.to_string())),
        },
        json_ld::Value::LangString(lang) => {
            let (value, language, direction) = lang.clone().into_parts();
            let mut obj = Map::new();
            obj.insert("@value".to_string(), JsonValue::String(value.to_string()));
            if let Some(lang_tag) = language {
                obj.insert(
                    "@language".to_string(),
                    JsonValue::String(lang_tag.to_string()),
                );
            }
            if let Some(dir) = direction {
                obj.insert("@direction".to_string(), JsonValue::String(dir.to_string()));
            }
            Some(JsonValue::Object(obj))
        }
        json_ld::Value::Json(json) => Some(json_syntax_value_to_json(json)),
    }
}

fn json_syntax_value_to_json(value: &SyntaxValue) -> JsonValue {
    match value {
        SyntaxValue::Null => JsonValue::Null,
        SyntaxValue::Boolean(b) => JsonValue::Bool(*b),
        SyntaxValue::Number(n) => serde_json::Number::from_str(&n.to_string())
            .ok()
            .map(JsonValue::Number)
            .unwrap_or_else(|| JsonValue::String(n.to_string())),
        SyntaxValue::String(s) => JsonValue::String(s.to_string()),
        SyntaxValue::Array(items) => {
            JsonValue::Array(items.iter().map(json_syntax_value_to_json).collect())
        }
        SyntaxValue::Object(obj) => {
            let mut map = Map::new();
            for entry in obj.iter() {
                map.insert(
                    entry.as_key().to_string(),
                    json_syntax_value_to_json(entry.as_value()),
                );
            }
            JsonValue::Object(map)
        }
    }
}

fn node_identifier(id: Option<&json_ld::Id<iref::IriBuf, rdf_types::BlankIdBuf>>) -> String {
    match id {
        Some(json_ld::Id::Valid(json_ld::ValidId::Iri(iri))) => iri.as_str().to_string(),
        Some(json_ld::Id::Valid(json_ld::ValidId::Blank(blank))) => {
            format!("_:{}", blank.as_str())
        }
        Some(json_ld::Id::Invalid(raw)) => raw.clone(),
        None => format!("_:{}", Uuid::new_v4()),
    }
}

fn id_to_string(id: &json_ld::Id<iref::IriBuf, rdf_types::BlankIdBuf>) -> String {
    match id {
        json_ld::Id::Valid(json_ld::ValidId::Iri(iri)) => iri.as_str().to_string(),
        json_ld::Id::Valid(json_ld::ValidId::Blank(blank)) => {
            format!("_:{}", blank.as_str())
        }
        json_ld::Id::Invalid(raw) => raw.clone(),
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
        if let Some(value) = resolve_node_property(&node.properties, key) {
            if let Some(text) = json_value_to_string(value) {
                return Some(text);
            }
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

    if let Some(last) = key.rsplit('/').next() {
        if let Some(value) = props.get(last) {
            return Some(value);
        }
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

fn render_variant_table(buf: &mut String, variants: &[VariantSummary], total_variants: usize) {
    if variants.is_empty() {
        return;
    }

    push_section_header(buf, "🧩", "Variants");

    let headers = [
        "SKU",
        "Color",
        "Size",
        "FrameShape",
        "Battery",
        "Price",
        "Availability",
    ];

    let mut rows: Vec<Vec<String>> = Vec::new();
    for variant in variants {
        rows.push(vec![
            variant.sku.clone().unwrap_or_else(|| "–".to_string()),
            variant.color.clone().unwrap_or_else(|| "–".to_string()),
            variant.size.clone().unwrap_or_else(|| "–".to_string()),
            variant
                .frame_shape
                .clone()
                .or_else(|| variant.additional.get("FrameShape").cloned())
                .unwrap_or_else(|| "–".to_string()),
            variant
                .battery
                .clone()
                .or_else(|| variant.additional.get("BatteryCapacity").cloned())
                .unwrap_or_else(|| "–".to_string()),
            variant
                .price_display
                .clone()
                .unwrap_or_else(|| "–".to_string()),
            variant
                .availability
                .as_ref()
                .map(|status| availability_label(status))
                .unwrap_or_else(|| "–".to_string()),
        ]);
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

    if let Some(last) = key.rsplit('/').next() {
        if let Some(value) = map.get(last) {
            return Some(value);
        }
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

fn sanitize_html_for_markdown(html: &str) -> String {
    static RE_TAG_BLOCKS: Lazy<Vec<Regex>> = Lazy::new(|| {
        [
            r"(?is)<script[^>]*?>[\s\S]*?</script>",
            r"(?is)<style[^>]*?>[\s\S]*?</style>",
            r"(?is)<noscript[^>]*?>[\s\S]*?</noscript>",
            r"(?is)<template[^>]*?>[\s\S]*?</template>",
        ]
        .into_iter()
        .map(|pattern| Regex::new(pattern).expect("invalid block regex"))
        .collect()
    });
    static RE_COMMENT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?is)<!--.*?-->").expect("invalid comment regex"));

    let mut clean = html.to_string();
    for re in RE_TAG_BLOCKS.iter() {
        clean = re.replace_all(&clean, "").into_owned();
    }

    RE_COMMENT.replace_all(&clean, "").into_owned()
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

    if let Some(query) = url.query() {
        if !query.is_empty() {
            parts.push(sanitize_for_filename(query));
        }
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
