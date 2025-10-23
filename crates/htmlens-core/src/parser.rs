//! HTML fetching and parsing functionality

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value as JsonValue;

/// Fetch HTML content from a URL
///
/// Requires the `full-expansion` feature (needs reqwest)
#[cfg(feature = "full-expansion")]
pub async fn fetch_html(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent(format!(
            "Mozilla/5.0 (compatible; htmlens-core/{})",
            env!("CARGO_PKG_VERSION")
        ))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to fetch URL")?;

    let html = response
        .text()
        .await
        .context("Failed to read response body")?;
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
                if text.is_empty() { None } else { Some(text) }
            } else {
                None
            }
        })
        .collect())
}

/// Combine multiple JSON-LD blocks into a single @graph structure
///
/// Uses the @context from the first block that has one. This works well when all blocks
/// use the same context (e.g., https://schema.org), which is the common case.
///
/// Handles both object and array top-level JSON-LD structures.
///
/// Note: If blocks use different contexts, this may cause incorrect term expansion.
/// For such cases, consider processing blocks separately or using a JSON-LD processor
/// that can handle multiple contexts correctly.
pub fn combine_json_ld_blocks(blocks: &[String]) -> Result<String> {
    if blocks.is_empty() {
        return Ok(r#"{"@context": "https://schema.org", "@graph": []}"#.to_string());
    }

    if blocks.len() == 1 {
        // Parse the single block to check if it has a context
        let parsed: JsonValue = serde_json::from_str(&blocks[0])
            .with_context(|| format!("failed to parse JSON-LD block: {}", blocks[0]))?;

        // If it already has a context, return as-is
        if parsed.get("@context").is_some() {
            return Ok(blocks[0].clone());
        }

        // If no context, add the default schema.org context
        if let JsonValue::Object(mut obj) = parsed {
            obj.insert(
                "@context".to_string(),
                JsonValue::String("https://schema.org".to_string()),
            );
            return Ok(serde_json::to_string(&obj)?);
        }

        // For non-objects, this is invalid JSON-LD
        return Err(anyhow::anyhow!(
            "Invalid JSON-LD: single block must be an object, got {:?}",
            parsed
        ));
    }

    // Parse each block and collect into a graph array
    let mut graph_items = Vec::new();
    let mut common_context = None;

    for block in blocks {
        let parsed: JsonValue = serde_json::from_str(block)
            .with_context(|| format!("failed to parse JSON-LD block: {}", block))?;

        // Use the @context from the first entry that has one
        if common_context.is_none()
            && let Some(ctx) = parsed.get("@context")
        {
            common_context = Some(ctx.clone());
        }

        // Handle both objects and arrays
        match parsed {
            JsonValue::Object(mut obj) => {
                // Remove @context from the item and add to graph
                obj.remove("@context");
                graph_items.push(JsonValue::Object(obj));
            }
            JsonValue::Array(arr) => {
                // Array at top level: add each object item to the graph
                for item in arr {
                    match item {
                        JsonValue::Object(mut obj) => {
                            obj.remove("@context");
                            graph_items.push(JsonValue::Object(obj));
                        }
                        _ => {
                            // Non-object items in arrays are unusual but valid JSON-LD
                            // (e.g., literal values). Include them as-is.
                            graph_items.push(item);
                        }
                    }
                }
            }
            _ => {
                // Top-level primitives (string, number, bool, null) are not valid JSON-LD documents
                return Err(anyhow::anyhow!(
                    "Invalid JSON-LD: top level must be an object or array, got {:?}",
                    parsed
                ));
            }
        }
    }

    // Build combined document with the context from the first entry
    // If no context was found in any block, use schema.org as default
    let context =
        common_context.unwrap_or_else(|| JsonValue::String("https://schema.org".to_string()));

    let combined = serde_json::json!({
        "@context": context,
        "@graph": graph_items
    });

    Ok(serde_json::to_string(&combined)?)
}

/// Sanitize HTML by removing script, style, and other unwanted elements
pub fn sanitize_html(html: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_ld_empty_html() {
        let html = "<html><body>No JSON-LD here</body></html>";
        let blocks = extract_json_ld_blocks(html).unwrap();
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_extract_json_ld_with_charset() {
        let html = r#"
            <script type="application/ld+json; charset=utf-8">
            {"@type": "Product", "name": "Test"}
            </script>
        "#;

        let blocks = extract_json_ld_blocks(html).unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("Test"));
    }

    #[test]
    fn test_extract_json_ld_case_insensitive() {
        let html = r#"
            <script type="APPLICATION/LD+JSON">
            {"@type": "Product", "name": "Test"}
            </script>
        "#;

        let blocks = extract_json_ld_blocks(html).unwrap();
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_extract_json_ld_empty_script() {
        let html = r#"
            <script type="application/ld+json"></script>
            <script type="application/ld+json">   </script>
        "#;

        let blocks = extract_json_ld_blocks(html).unwrap();
        assert!(blocks.is_empty()); // Empty scripts should be filtered out
    }

    #[test]
    fn test_combine_single_block() {
        let blocks = vec![
            r#"{"@context": "https://schema.org", "@type": "Product", "name": "Single"}"#
                .to_string(),
        ];

        let combined = combine_json_ld_blocks(&blocks).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&combined).unwrap();

        // For single block, the function returns the original object (not @graph structure)
        assert_eq!(parsed["@context"], "https://schema.org");
        assert_eq!(parsed["@type"], "Product");
        assert_eq!(parsed["name"], "Single");
    }

    #[test]
    fn test_combine_multiple_blocks() {
        let blocks = vec![
            r#"{"@context": "https://schema.org", "@type": "Product", "name": "Product1"}"#
                .to_string(),
            r#"{"@type": "Organization", "name": "Org1"}"#.to_string(),
        ];

        let combined = combine_json_ld_blocks(&blocks).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&combined).unwrap();

        // For multiple blocks, creates @graph structure
        assert_eq!(parsed["@context"], "https://schema.org");
        assert!(parsed["@graph"].is_array());
        assert_eq!(parsed["@graph"].as_array().unwrap().len(), 2);

        // Check that contexts are removed from graph items
        let graph_items = parsed["@graph"].as_array().unwrap();
        assert_eq!(graph_items[0]["@type"], "Product");
        assert_eq!(graph_items[0]["name"], "Product1");
        assert!(graph_items[0].get("@context").is_none());
    }

    #[test]
    fn test_combine_blocks_no_context() {
        let blocks = vec![r#"{"@type": "Product", "name": "No Context"}"#.to_string()];

        let combined = combine_json_ld_blocks(&blocks).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&combined).unwrap();

        // For single block without context, should use default context
        assert_eq!(parsed["@context"], "https://schema.org");
        assert_eq!(parsed["@type"], "Product");
        assert_eq!(parsed["name"], "No Context");
    }

    #[test]
    fn test_combine_empty_blocks() {
        let blocks: Vec<String> = vec![];

        let combined = combine_json_ld_blocks(&blocks).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&combined).unwrap();

        assert_eq!(parsed["@context"], "https://schema.org");
        assert_eq!(parsed["@graph"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_combine_invalid_json() {
        let blocks = vec![
            r#"{"@type": "Product""#.to_string(), // Invalid JSON
            r#"{"@type": "Organization", "name": "Valid"}"#.to_string(),
        ];

        // Should return an error for invalid JSON (the function uses with_context for errors)
        let result = combine_json_ld_blocks(&blocks);
        assert!(result.is_err());

        // Test with only valid JSON (single block)
        let valid_blocks = vec![r#"{"@type": "Organization", "name": "Valid"}"#.to_string()];

        let combined = combine_json_ld_blocks(&valid_blocks).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&combined).unwrap();

        // Single block should be returned as an object with added context, not @graph
        assert_eq!(parsed["@context"], "https://schema.org");
        assert_eq!(parsed["@type"], "Organization");
        assert_eq!(parsed["name"], "Valid");
    }

    #[test]
    fn test_sanitize_html_removes_scripts() {
        let html = r#"
            <p>Keep this</p>
            <script>alert('remove this')</script>
            <script src="external.js"></script>
        "#;

        let sanitized = sanitize_html(html);
        assert!(sanitized.contains("Keep this"));
        assert!(!sanitized.contains("script"));
        assert!(!sanitized.contains("alert"));
    }

    #[test]
    fn test_sanitize_html_removes_styles() {
        let html = r#"
            <p>Keep this</p>
            <style>body { background: red; }</style>
            <link rel="stylesheet" href="style.css">
        "#;

        let sanitized = sanitize_html(html);
        assert!(sanitized.contains("Keep this"));
        assert!(!sanitized.contains("<style"));
        assert!(!sanitized.contains("background: red"));
    }

    #[test]
    fn test_sanitize_html_removes_comments() {
        let html = r#"
            <p>Keep this</p>
            <!-- This is a comment -->
            <div><!-- Another comment --></div>
        "#;

        let sanitized = sanitize_html(html);
        assert!(sanitized.contains("Keep this"));
        assert!(!sanitized.contains("<!--"));
        assert!(!sanitized.contains("comment"));
    }

    #[test]
    fn test_html_to_markdown_headings() {
        let html = r#"
            <h1>Heading 1</h1>
            <h2>Heading 2</h2>
            <h3>Heading 3</h3>
        "#;

        let markdown = html_to_markdown(html);
        // html2md uses alternative heading formats for h1 and h2
        assert!(markdown.contains("Heading 1\n=========="));
        assert!(markdown.contains("Heading 2\n----------"));
        assert!(markdown.contains("### Heading 3 ###"));
    }

    #[test]
    fn test_html_to_markdown_links() {
        let html = r#"<a href="https://example.com">Link text</a>"#;
        let markdown = html_to_markdown(html);
        assert!(markdown.contains("[Link text](https://example.com)"));
    }

    #[test]
    fn test_html_to_markdown_emphasis() {
        let html = r#"
            <p>Text with <strong>bold</strong> and <em>italic</em>.</p>
        "#;

        let markdown = html_to_markdown(html);
        assert!(markdown.contains("**bold**"));
        assert!(markdown.contains("*italic*"));
    }
}
