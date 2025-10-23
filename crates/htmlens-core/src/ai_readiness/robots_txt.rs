//! Robots.txt parser and analyzer
//!
//! Parses robots.txt files and analyzes crawling rules for different user agents,
//! with special focus on AI crawlers (GPTBot, ClaudeBot, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Known AI crawler user agents
pub const AI_CRAWLERS: &[&str] = &[
    "GPTBot",          // OpenAI ChatGPT
    "ChatGPT-User",    // OpenAI ChatGPT browsing
    "ClaudeBot",       // Anthropic Claude
    "Claude-Web",      // Anthropic Claude web
    "Google-Extended", // Google Bard/Gemini
    "Bingbot",         // Microsoft Bing
    "Applebot",        // Apple Siri
    "Anthropic-AI",    // Anthropic general
    "PerplexityBot",   // Perplexity AI
    "YouBot",          // You.com AI
];

/// Results from parsing robots.txt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsTxtAnalysis {
    /// Whether robots.txt was found
    pub found: bool,

    /// HTTP status code
    pub status_code: u16,

    /// Raw content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Sitemap URLs found
    pub sitemaps: Vec<String>,

    /// Rules per user agent
    pub agent_rules: HashMap<String, AgentRules>,

    /// AI crawler analysis
    pub ai_crawler_status: Vec<AiCrawlerStatus>,

    /// Parsing issues
    pub issues: Vec<String>,
}

/// Rules for a specific user agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRules {
    /// User agent name
    pub user_agent: String,

    /// Disallowed paths
    pub disallow: Vec<String>,

    /// Explicitly allowed paths
    pub allow: Vec<String>,

    /// Crawl delay in seconds
    pub crawl_delay: Option<u32>,

    /// Whether this blocks the entire site
    pub blocks_all: bool,
}

/// Status for a specific AI crawler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCrawlerStatus {
    /// Crawler name
    pub name: String,

    /// Access level
    pub access: AccessLevel,

    /// Specific rules that apply
    pub applicable_rules: Option<String>,
}

/// Access level for a crawler
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    /// Full access to site
    Allowed,

    /// Partially blocked (some paths disallowed)
    Partial,

    /// Fully blocked
    Blocked,

    /// No specific rules (inherits from *)
    Default,
}

impl Default for RobotsTxtAnalysis {
    fn default() -> Self {
        Self {
            found: false,
            status_code: 404,
            content: None,
            sitemaps: Vec::new(),
            agent_rules: HashMap::new(),
            ai_crawler_status: Vec::new(),
            issues: Vec::new(),
        }
    }
}

/// Parse robots.txt content
pub fn parse_robots_txt(content: &str) -> RobotsTxtAnalysis {
    let mut analysis = RobotsTxtAnalysis {
        found: true,
        status_code: 200,
        content: Some(content.to_string()),
        ..Default::default()
    };

    let mut current_agents: Vec<String> = Vec::new();
    let mut current_rules = AgentRules {
        user_agent: String::new(),
        disallow: Vec::new(),
        allow: Vec::new(),
        crawl_delay: None,
        blocks_all: false,
    };

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split on first colon
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }

        let directive = parts[0].trim().to_lowercase();
        let value = parts[1].trim();

        match directive.as_str() {
            "user-agent" => {
                // Save previous agent rules if any
                if !current_agents.is_empty() {
                    for agent in &current_agents {
                        analysis
                            .agent_rules
                            .insert(agent.clone(), current_rules.clone());
                    }
                }

                // Start new agent
                current_agents = vec![value.to_string()];
                current_rules = AgentRules {
                    user_agent: value.to_string(),
                    disallow: Vec::new(),
                    allow: Vec::new(),
                    crawl_delay: None,
                    blocks_all: false,
                };
            }
            "disallow" => {
                if !value.is_empty() {
                    current_rules.disallow.push(value.to_string());
                    if value == "/" {
                        current_rules.blocks_all = true;
                    }
                }
            }
            "allow" => {
                if !value.is_empty() {
                    current_rules.allow.push(value.to_string());
                }
            }
            "crawl-delay" => {
                if let Ok(delay) = value.parse::<u32>() {
                    current_rules.crawl_delay = Some(delay);
                }
            }
            "sitemap" => {
                if !value.is_empty() {
                    analysis.sitemaps.push(value.to_string());
                }
            }
            _ => {
                // Unknown directive - could add to issues
            }
        }
    }

    // Save last agent rules
    if !current_agents.is_empty() {
        for agent in &current_agents {
            analysis
                .agent_rules
                .insert(agent.clone(), current_rules.clone());
        }
    }

    // Analyze AI crawlers
    analysis.ai_crawler_status = analyze_ai_crawlers(&analysis.agent_rules);

    // Validate and add issues
    if analysis.sitemaps.is_empty() {
        analysis.issues.push("No sitemap URLs found".to_string());
    }

    if let Some(wildcard) = analysis.agent_rules.get("*")
        && wildcard.blocks_all
    {
        analysis
            .issues
            .push("Warning: All bots blocked with 'Disallow: /'".to_string());
    }

    analysis
}

/// Analyze access for known AI crawlers
fn analyze_ai_crawlers(agent_rules: &HashMap<String, AgentRules>) -> Vec<AiCrawlerStatus> {
    let mut statuses = Vec::new();

    for crawler in AI_CRAWLERS {
        let access = determine_access(crawler, agent_rules);
        let applicable_rules = get_applicable_rules(crawler, agent_rules);

        statuses.push(AiCrawlerStatus {
            name: crawler.to_string(),
            access,
            applicable_rules,
        });
    }

    statuses
}

/// Determine access level for a specific crawler
fn determine_access(crawler: &str, agent_rules: &HashMap<String, AgentRules>) -> AccessLevel {
    // Check for exact match
    if let Some(rules) = agent_rules.get(crawler) {
        if rules.blocks_all {
            return AccessLevel::Blocked;
        }
        if !rules.disallow.is_empty() {
            return AccessLevel::Partial;
        }
        return AccessLevel::Allowed;
    }

    // Check for case-insensitive match
    let crawler_lower = crawler.to_lowercase();
    for (agent, rules) in agent_rules {
        if agent.to_lowercase() == crawler_lower {
            if rules.blocks_all {
                return AccessLevel::Blocked;
            }
            if !rules.disallow.is_empty() {
                return AccessLevel::Partial;
            }
            return AccessLevel::Allowed;
        }
    }

    // Fall back to wildcard rules
    if let Some(rules) = agent_rules.get("*") {
        if rules.blocks_all {
            return AccessLevel::Blocked;
        }
        if !rules.disallow.is_empty() {
            return AccessLevel::Partial;
        }
    }

    AccessLevel::Default
}

/// Get description of applicable rules
fn get_applicable_rules(
    crawler: &str,
    agent_rules: &HashMap<String, AgentRules>,
) -> Option<String> {
    // Check for exact match
    if let Some(rules) = agent_rules.get(crawler) {
        return Some(format_rules(crawler, rules));
    }

    // Check for case-insensitive match
    let crawler_lower = crawler.to_lowercase();
    for (agent, rules) in agent_rules {
        if agent.to_lowercase() == crawler_lower {
            return Some(format_rules(agent, rules));
        }
    }

    // Fall back to wildcard
    if let Some(rules) = agent_rules.get("*") {
        return Some(format_rules("* (all bots)", rules));
    }

    None
}

/// Format rules into human-readable string
fn format_rules(agent: &str, rules: &AgentRules) -> String {
    let mut parts = vec![format!("User-agent: {}", agent)];

    if rules.blocks_all {
        parts.push("Disallow: / (FULL BLOCK)".to_string());
    } else {
        if !rules.disallow.is_empty() {
            for path in &rules.disallow {
                parts.push(format!("Disallow: {}", path));
            }
        }
        if !rules.allow.is_empty() {
            for path in &rules.allow {
                parts.push(format!("Allow: {}", path));
            }
        }
    }

    if let Some(delay) = rules.crawl_delay {
        parts.push(format!("Crawl-delay: {}s", delay));
    }

    parts.join("; ")
}

/// Check if a specific path is allowed for a user agent
pub fn is_path_allowed(path: &str, agent: &str, analysis: &RobotsTxtAnalysis) -> bool {
    let rules = analysis
        .agent_rules
        .get(agent)
        .or_else(|| analysis.agent_rules.get("*"));

    if let Some(rules) = rules {
        // Check allow rules first (they take precedence)
        for allow_path in &rules.allow {
            if path.starts_with(allow_path) {
                return true;
            }
        }

        // Check disallow rules
        for disallow_path in &rules.disallow {
            if disallow_path == "/" {
                return false; // Blocked entirely
            }
            if path.starts_with(disallow_path) {
                return false;
            }
        }
    }

    true // Default is allowed
}

#[cfg(all(test, feature = "ai-readiness"))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_robots() {
        let content = r#"
User-agent: *
Disallow: /admin/
Disallow: /private/

Sitemap: https://example.com/sitemap.xml
"#;

        let analysis = parse_robots_txt(content);
        assert!(analysis.found);
        assert_eq!(analysis.sitemaps.len(), 1);
        assert!(analysis.agent_rules.contains_key("*"));

        let wildcard = &analysis.agent_rules["*"];
        assert_eq!(wildcard.disallow.len(), 2);
        assert!(!wildcard.blocks_all);
    }

    #[test]
    fn test_parse_ai_crawler_block() {
        let content = r#"
User-agent: *
Disallow:

User-agent: GPTBot
Disallow: /
"#;

        let analysis = parse_robots_txt(content);

        let gptbot_status = analysis
            .ai_crawler_status
            .iter()
            .find(|s| s.name == "GPTBot")
            .unwrap();

        assert_eq!(gptbot_status.access, AccessLevel::Blocked);
    }

    #[test]
    fn test_is_path_allowed() {
        let content = r#"
User-agent: *
Disallow: /admin/
Allow: /admin/public/
"#;

        let analysis = parse_robots_txt(content);

        assert!(is_path_allowed("/", "*", &analysis));
        assert!(!is_path_allowed("/admin/secret", "*", &analysis));
        assert!(is_path_allowed("/admin/public/doc.html", "*", &analysis));
    }

    #[test]
    fn test_multiple_sitemaps() {
        let content = r#"
Sitemap: https://example.com/sitemap.xml
Sitemap: https://example.com/sitemap-images.xml
Sitemap: https://example.com/sitemap-videos.xml
"#;

        let analysis = parse_robots_txt(content);
        assert_eq!(analysis.sitemaps.len(), 3);
    }
}
