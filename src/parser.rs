//! HTML fetching and parsing functionality

use anyhow::{Context, Result};
use scraper::{Html, Selector};

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

/// Convert HTML to Markdown
pub fn html_to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

/// Extract JSON-LD script blocks from HTML
pub fn extract_json_ld_blocks(html: &str) -> Result<Vec<String>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("script[type='application/ld+json']")
        .map_err(|e| anyhow::anyhow!("Invalid selector: {:?}", e))?;

    let blocks: Vec<String> = document
        .select(&selector)
        .filter_map(|el| {
            let text = el.text().collect::<String>();
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        })
        .collect();

    Ok(blocks)
}
