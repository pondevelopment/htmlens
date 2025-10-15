//! HTML fetching and parsing functionality

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value as JsonValue;

/// Fetch HTML content from a URL
pub async fn fetch_html(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent(&format!("Mozilla/5.0 (compatible; htmlens/{})", env!("CARGO_PKG_VERSION")))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to fetch URL")?;

    let html = response.text().await.context("Failed to read response body")?;
    Ok(html)
}

/// Extract JSON-LD script blocks from HTML
pub fn extract_json_ld_blocks(html: &str) -> Result<Vec<String>> {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script")
        .map_err(|e| anyhow::anyhow!("unable to parse selector: {}", e))?;

    Ok(document
        .select(&script_selector)
        .filter_map(|element| {
            let script_type = element
                .value()
                .attr("type")
                .map(|t| t.trim().to_ascii_lowercase())
                .unwrap_or_default();

            // Use contains() to catch variations like "application/ld+json; charset=utf-8"
            if script_type.contains("ld+json") {
                let text = element.text().collect::<String>().trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            } else {
                None
            }
        })
        .collect())
}

/// Combine multiple JSON-LD blocks into a single @graph structure
pub fn combine_json_ld_blocks(blocks: &[String]) -> Result<String> {
    if blocks.is_empty() {
        return Ok(r#"{"@context": "https://schema.org", "@graph": []}"#.to_string());
    }

    if blocks.len() == 1 {
        return Ok(blocks[0].clone());
    }

    // Parse each block and collect into a graph array
    let mut graph_items = Vec::new();
    let mut common_context = None;

    for block in blocks {
        let parsed: JsonValue = serde_json::from_str(block)
            .with_context(|| format!("failed to parse JSON-LD block: {}", block))?;

        // Use the @context from the first entry that has one
        if common_context.is_none() {
            if let Some(ctx) = parsed.get("@context") {
                common_context = Some(ctx.clone());
            }
        }

        // Remove @context from the item and add to graph
        if let JsonValue::Object(mut obj) = parsed {
            obj.remove("@context");
            graph_items.push(JsonValue::Object(obj));
        }
    }

    // Build combined document with the context from the first entry
    // If no context was found in any block, we have to use a default
    let context = common_context.unwrap_or_else(|| {
        JsonValue::String("https://schema.org".to_string())
    });

    let combined = serde_json::json!({
        "@context": context,
        "@graph": graph_items
    });

    Ok(serde_json::to_string(&combined)?)
}

/// Sanitize HTML by removing script, style, and other unwanted elements
fn sanitize_html(html: &str) -> String {
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

/// Convert HTML to Markdown (with sanitization)
pub fn html_to_markdown(html: &str) -> String {
    let sanitized = sanitize_html(html);
    html2md::parse_html(&sanitized)
}
