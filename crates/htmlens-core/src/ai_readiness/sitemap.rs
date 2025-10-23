use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Analysis results for XML sitemap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapAnalysis {
    pub found: bool,
    pub sitemap_type: SitemapType,
    pub url_count: usize,
    pub last_modified: Option<String>,
    pub url_entries: Vec<SitemapUrl>,
    pub statistics: SitemapStatistics,
    pub nested_sitemaps: Vec<String>,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SitemapType {
    Standard, // Single sitemap with URLs
    Index,    // Sitemap index pointing to other sitemaps
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapUrl {
    pub loc: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapStatistics {
    pub total_urls: usize,
    pub urls_with_lastmod: usize,
    pub urls_with_priority: usize,
    pub avg_priority: f32,
    pub content_types: HashMap<String, usize>, // e.g., "product", "article", "page"
}

/// Parse XML sitemap content
pub fn parse_sitemap(content: &str, base_domain: &str) -> Result<SitemapAnalysis> {
    let mut analysis = SitemapAnalysis {
        found: true,
        sitemap_type: SitemapType::Unknown,
        url_count: 0,
        last_modified: None,
        url_entries: Vec::new(),
        statistics: SitemapStatistics {
            total_urls: 0,
            urls_with_lastmod: 0,
            urls_with_priority: 0,
            avg_priority: 0.0,
            content_types: HashMap::new(),
        },
        nested_sitemaps: Vec::new(),
        issues: Vec::new(),
        recommendations: Vec::new(),
    };

    // Check if it's a sitemap index
    if content.contains("<sitemapindex") {
        analysis.sitemap_type = SitemapType::Index;
        parse_sitemap_index(content, &mut analysis)?;
    } else if content.contains("<urlset") {
        analysis.sitemap_type = SitemapType::Standard;
        parse_urlset(content, base_domain, &mut analysis)?;
    } else {
        analysis
            .issues
            .push("Invalid sitemap format - missing <urlset> or <sitemapindex>".to_string());
        return Ok(analysis);
    }

    // Validate and add recommendations
    validate_sitemap(&mut analysis, base_domain);

    Ok(analysis)
}

fn parse_sitemap_index(content: &str, analysis: &mut SitemapAnalysis) -> Result<()> {
    // Extract sitemap locations from index - use DOTALL flag for multiline matching
    let sitemap_pattern = regex::Regex::new(r"(?s)<sitemap>.*?<loc>(.*?)</loc>.*?</sitemap>")
        .context("Failed to create regex")?;

    for cap in sitemap_pattern.captures_iter(content) {
        if let Some(loc) = cap.get(1) {
            let url = decode_xml_entities(loc.as_str().trim());
            analysis.nested_sitemaps.push(url);
        }
    }

    analysis.url_count = analysis.nested_sitemaps.len();

    if analysis.nested_sitemaps.is_empty() {
        analysis
            .issues
            .push("Sitemap index contains no nested sitemaps".to_string());
    }

    Ok(())
}

fn parse_urlset(content: &str, base_domain: &str, analysis: &mut SitemapAnalysis) -> Result<()> {
    // Parse URL entries
    let url_pattern = regex::Regex::new(
        r"(?s)<url>.*?<loc>(.*?)</loc>(?:.*?<lastmod>(.*?)</lastmod>)?(?:.*?<changefreq>(.*?)</changefreq>)?(?:.*?<priority>(.*?)</priority>)?.*?</url>"
    ).context("Failed to create regex")?;

    let mut priority_sum: f32 = 0.0;

    for cap in url_pattern.captures_iter(content) {
        let loc = cap
            .get(1)
            .map(|m| decode_xml_entities(m.as_str().trim()))
            .unwrap_or_default();
        let lastmod = cap.get(2).map(|m| m.as_str().trim().to_string());
        let changefreq = cap.get(3).map(|m| m.as_str().trim().to_string());
        let priority = cap
            .get(4)
            .and_then(|m| m.as_str().trim().parse::<f32>().ok());

        // Check domain mismatch
        if !loc.is_empty() && !loc.starts_with(base_domain) {
            analysis
                .issues
                .push(format!("URL on wrong domain: {}", loc));
        }

        // Validate priority range
        if let Some(p) = priority {
            if !(0.0..=1.0).contains(&p) {
                analysis
                    .issues
                    .push(format!("Invalid priority {} for URL: {}", p, loc));
            } else {
                priority_sum += p;
                analysis.statistics.urls_with_priority += 1;
            }
        }

        // Count lastmod
        if lastmod.is_some() {
            analysis.statistics.urls_with_lastmod += 1;
        }

        // Categorize content type
        let content_type = categorize_url(&loc);
        *analysis
            .statistics
            .content_types
            .entry(content_type)
            .or_insert(0) += 1;

        analysis.url_entries.push(SitemapUrl {
            loc,
            lastmod,
            changefreq,
            priority,
        });
    }

    analysis.url_count = analysis.url_entries.len();
    analysis.statistics.total_urls = analysis.url_entries.len();

    if analysis.statistics.urls_with_priority > 0 {
        analysis.statistics.avg_priority =
            priority_sum / analysis.statistics.urls_with_priority as f32;
    }

    Ok(())
}

fn validate_sitemap(analysis: &mut SitemapAnalysis, _base_domain: &str) {
    // Check URL count limits
    if analysis.sitemap_type == SitemapType::Standard && analysis.url_count > 50_000 {
        analysis.issues.push(format!(
            "Sitemap exceeds 50,000 URL limit ({} URLs) - consider using sitemap index",
            analysis.url_count
        ));
    }

    if analysis.url_count == 0 {
        analysis.issues.push("Sitemap contains no URLs".to_string());
    }

    // Check for missing metadata
    let lastmod_percentage = if analysis.statistics.total_urls > 0 {
        (analysis.statistics.urls_with_lastmod as f32 / analysis.statistics.total_urls as f32)
            * 100.0
    } else {
        0.0
    };

    if lastmod_percentage < 50.0 && analysis.statistics.total_urls > 0 {
        analysis.recommendations.push(format!(
            "Only {:.0}% of URLs have lastmod dates - consider adding them for better crawl efficiency",
            lastmod_percentage
        ));
    }

    // Check priority usage
    if analysis.statistics.urls_with_priority == 0 && analysis.statistics.total_urls > 0 {
        analysis
            .recommendations
            .push("No priority values set - consider using priority to guide crawlers".to_string());
    }

    // Check for AI-relevant content types
    let ai_relevant_types = ["article", "blog", "product", "documentation"];
    let has_ai_content = ai_relevant_types
        .iter()
        .any(|t| analysis.statistics.content_types.contains_key(*t));

    if has_ai_content {
        let mut relevant_counts = Vec::new();
        for content_type in &ai_relevant_types {
            if let Some(count) = analysis.statistics.content_types.get(*content_type) {
                relevant_counts.push(format!("{}: {}", content_type, count));
            }
        }
        if !relevant_counts.is_empty() {
            analysis.recommendations.push(format!(
                "AI-relevant content found: {}",
                relevant_counts.join(", ")
            ));
        }
    }
}

fn categorize_url(url: &str) -> String {
    let url_lower = url.to_lowercase();

    // Common patterns for content types
    if url_lower.contains("/product") || url_lower.contains("/item") || url_lower.contains("/shop")
    {
        "product".to_string()
    } else if url_lower.contains("/blog") || url_lower.contains("/post") {
        "blog".to_string()
    } else if url_lower.contains("/article") || url_lower.contains("/news") {
        "article".to_string()
    } else if url_lower.contains("/doc")
        || url_lower.contains("/guide")
        || url_lower.contains("/tutorial")
    {
        "documentation".to_string()
    } else if url_lower.contains("/video") || url_lower.contains("/watch") {
        "video".to_string()
    } else if url_lower.contains("/image") || url_lower.contains("/gallery") {
        "image".to_string()
    } else if url_lower.contains("/faq") || url_lower.contains("/help") {
        "faq".to_string()
    } else if url_lower.contains("/about")
        || url_lower.contains("/contact")
        || url_lower.contains("/privacy")
    {
        "info".to_string()
    } else {
        "page".to_string()
    }
}

fn decode_xml_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(all(test, feature = "ai-readiness"))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_sitemap() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/page1</loc>
    <lastmod>2025-10-01</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>https://example.com/page2</loc>
    <priority>0.5</priority>
  </url>
</urlset>"#;

        let result = parse_sitemap(xml, "https://example.com").unwrap();
        assert_eq!(result.sitemap_type, SitemapType::Standard);
        assert_eq!(result.url_count, 2);
        assert_eq!(result.statistics.urls_with_lastmod, 1);
        assert_eq!(result.statistics.urls_with_priority, 2);
        assert!(result.statistics.avg_priority > 0.0);
    }

    #[test]
    fn test_parse_sitemap_index() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://example.com/sitemap1.xml</loc>
  </sitemap>
  <sitemap>
    <loc>https://example.com/sitemap2.xml</loc>
  </sitemap>
</sitemapindex>"#;

        let result = parse_sitemap(xml, "https://example.com").unwrap();
        assert_eq!(result.sitemap_type, SitemapType::Index);
        assert_eq!(result.nested_sitemaps.len(), 2);
    }

    #[test]
    fn test_domain_validation() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/page1</loc>
  </url>
  <url>
    <loc>https://other-domain.com/page2</loc>
  </url>
</urlset>"#;

        let result = parse_sitemap(xml, "https://example.com").unwrap();
        assert!(result.issues.iter().any(|i| i.contains("wrong domain")));
    }

    #[test]
    fn test_categorize_urls() {
        assert_eq!(
            categorize_url("https://example.com/products/item123"),
            "product"
        );
        assert_eq!(categorize_url("https://example.com/blog/my-post"), "blog");
        assert_eq!(
            categorize_url("https://example.com/docs/guide"),
            "documentation"
        );
        assert_eq!(categorize_url("https://example.com/about"), "info");
    }

    #[test]
    fn test_xml_entity_decoding() {
        let encoded = "https://example.com/page?param=1&amp;other=2";
        let decoded = decode_xml_entities(encoded);
        assert_eq!(decoded, "https://example.com/page?param=1&other=2");
    }

    #[test]
    fn test_priority_validation() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/page1</loc>
    <priority>1.5</priority>
  </url>
</urlset>"#;

        let result = parse_sitemap(xml, "https://example.com").unwrap();
        assert!(result.issues.iter().any(|i| i.contains("Invalid priority")));
    }

    #[test]
    fn test_url_limit_warning() {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
        );

        // Add 50,001 URLs
        for i in 0..50_001 {
            xml.push_str(&format!(
                r#"<url><loc>https://example.com/page{}</loc></url>"#,
                i
            ));
        }
        xml.push_str("</urlset>");

        let result = parse_sitemap(&xml, "https://example.com").unwrap();
        assert!(result.issues.iter().any(|i| i.contains("exceeds 50,000")));
    }
}
