use crate::url_utils::normalize_origin;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use url::Url;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WebMcpConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebMcpProbe {
    pub path: String,
    pub status: u16,
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct WebMcpTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebMcpScanResult {
    pub supported: bool,
    pub confidence: WebMcpConfidence,
    pub score: u8,
    pub signals: Vec<String>,
    #[serde(rename = "manifestUrl", skip_serializing_if = "Option::is_none")]
    pub manifest_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(rename = "protocolVersion", skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(rename = "outdatedDetails")]
    pub outdated_details: Vec<String>,
    #[serde(rename = "availableRoutes")]
    pub available_routes: Vec<String>,
    #[serde(rename = "availableTools")]
    pub available_tools: Vec<WebMcpTool>,
    pub probes: Vec<WebMcpProbe>,
}

impl Default for WebMcpScanResult {
    fn default() -> Self {
        Self {
            supported: false,
            confidence: WebMcpConfidence::Low,
            score: 0,
            signals: Vec::new(),
            manifest_url: None,
            endpoint: None,
            protocol_version: None,
            outdated_details: Vec::new(),
            available_routes: Vec::new(),
            available_tools: Vec::new(),
            probes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WebMcpEvidence {
    pub manifest_found: bool,
    pub manifest_valid: bool,
    pub server_card_found: bool,
    pub server_card_valid: bool,
    pub server_cards_found: bool,
    pub server_cards_valid: bool,
    pub html_marker_count: usize,
    pub link_hint_count: usize,
    pub endpoint_found: bool,
    pub endpoint_probe_success: bool,
    pub endpoint_probe_from_manifest_success: bool,
    pub protocol_version_found: bool,
    pub in_page_model_context_api: bool,
    pub in_page_register_tool_usage: bool,
    pub in_page_tool_schema_usage: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WebMcpCandidate {
    pub valid_structure: bool,
    pub endpoint: Option<String>,
    pub protocol_version: Option<String>,
    pub has_name: bool,
    pub has_transport: bool,
    pub has_capabilities: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WebMcpHtmlSignals {
    pub signals: Vec<String>,
    pub has_model_context_api: bool,
    pub has_register_tool_usage: bool,
    pub has_tool_schema_usage: bool,
}

const RECOMMENDED_PROTOCOL_VERSION: (u32, u32, u32) = (2025, 6, 18);

pub fn analyze_web_mcp_html_signals(html: &str) -> WebMcpHtmlSignals {
    let lower = html.to_lowercase();
    let mut out = WebMcpHtmlSignals::default();

    if lower.contains("/.well-known/mcp.json") {
        out.signals
            .push("Homepage references /.well-known/mcp.json".to_string());
    }
    if lower.contains("/.well-known/mcp/server-card.json")
        || lower.contains("/.well-known/mcp/server-cards.json")
    {
        out.signals
            .push("Homepage references WebMCP server-card files".to_string());
    }
    if lower.contains("modelcontextprotocol") || lower.contains("webmcp") {
        out.signals
            .push("Homepage mentions Model Context Protocol / WebMCP".to_string());
    }
    if lower.contains("rel=\"mcp\"")
        || lower.contains("name=\"mcp\"")
        || lower.contains("type=\"application/mcp+json\"")
    {
        out.signals
            .push("Homepage includes MCP-specific meta/link/script markers".to_string());
    }

    if lower.contains("document.modelcontext")
        || lower.contains("window.modelcontext")
        || lower.contains(".modelcontext.")
    {
        out.has_model_context_api = true;
        out.signals
            .push("In-page WebMCP API detected (modelContext)".to_string());
    }
    if lower.contains("registertool(") || lower.contains(".registertool(") {
        out.has_register_tool_usage = true;
        out.signals
            .push("In-page tool registration detected (registerTool)".to_string());
    }
    if lower.contains("inputschema") || lower.contains("\"inputschema\"") {
        out.has_tool_schema_usage = true;
        out.signals
            .push("In-page tool schema detected (inputSchema)".to_string());
    }

    out
}

pub fn detect_web_mcp_html_markers(html: &str) -> Vec<String> {
    analyze_web_mcp_html_signals(html).signals
}

pub fn detect_web_mcp_link_hints(link_header: &str) -> Vec<String> {
    if link_header.trim().is_empty() {
        return Vec::new();
    }

    let lower = link_header.to_lowercase();
    let mut signals = Vec::new();

    if lower.contains("rel=\"mcp\"") || lower.contains(" rel=mcp") {
        signals.push("Link header exposes rel=mcp".to_string());
    }
    if lower.contains("/.well-known/mcp.json")
        || lower.contains("/.well-known/mcp/server-card.json")
        || lower.contains("/.well-known/mcp/server-cards.json")
    {
        signals.push("Link header points to WebMCP well-known resources".to_string());
    }
    if lower.contains("modelcontextprotocol") || lower.contains("webmcp") {
        signals.push("Link header contains MCP/WebMCP hints".to_string());
    }

    signals
}

pub fn parse_web_mcp_candidate(
    json: &JsonValue,
    origin_or_base_url: &str,
    source_path: &str,
) -> WebMcpCandidate {
    let node = if source_path.ends_with("server-cards.json") {
        json.as_array()
            .and_then(|arr| arr.first())
            .cloned()
            .unwrap_or_else(|| json.clone())
    } else {
        json.clone()
    };

    let has_name = node
        .get("name")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty());

    let protocol_version = node
        .get("protocolVersion")
        .or_else(|| node.get("protocol_version"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let endpoint_raw = node
        .get("transport")
        .and_then(|v| v.get("endpoint"))
        .and_then(|v| v.as_str())
        .or_else(|| node.get("endpoint").and_then(|v| v.as_str()))
        .or_else(|| node.get("url").and_then(|v| v.as_str()));

    let endpoint = endpoint_raw
        .map(str::to_string)
        .and_then(|raw| absolutize_endpoint(&raw, origin_or_base_url));

    let has_transport = node.get("transport").is_some() || endpoint.is_some();
    let has_capabilities = node.get("capabilities").is_some();

    let valid_structure = has_name
        && protocol_version.is_some()
        && endpoint.is_some()
        && (has_transport || has_capabilities);

    WebMcpCandidate {
        valid_structure,
        endpoint,
        protocol_version,
        has_name,
        has_transport,
        has_capabilities,
    }
}

pub fn detect_outdated_web_mcp_details(json: &JsonValue) -> Vec<String> {
    let mut findings = Vec::new();
    let mut all_strings = Vec::new();
    collect_string_values(json, &mut all_strings);

    for value in &all_strings {
        let lower = value.to_lowercase();
        if lower.contains("anthropic.com/news/agent-skills")
            || lower.contains("anthropic.com/docs/claude")
        {
            findings.push(format!(
                "Legacy Anthropic MCP reference detected ({value}); prefer https://modelcontextprotocol.io/"
            ));
        }
    }

    for protocol_version in extract_protocol_versions(json) {
        if let Some(detail) = detect_outdated_protocol_version(&protocol_version) {
            findings.push(detail);
        }
    }

    findings.sort();
    findings.dedup();
    findings
}

pub fn detect_outdated_protocol_version(protocol_version: &str) -> Option<String> {
    let parsed = parse_protocol_date(protocol_version)?;
    if parsed < RECOMMENDED_PROTOCOL_VERSION {
        return Some(format!(
            "Outdated protocolVersion '{protocol_version}' detected; expected {}-{:02}-{:02} or newer",
            RECOMMENDED_PROTOCOL_VERSION.0,
            RECOMMENDED_PROTOCOL_VERSION.1,
            RECOMMENDED_PROTOCOL_VERSION.2
        ));
    }
    None
}

pub fn score_web_mcp_evidence(e: &WebMcpEvidence) -> (u8, WebMcpConfidence, bool) {
    let mut score: u16 = 0;

    if e.manifest_valid {
        score += 45;
    } else if e.manifest_found {
        score += 15;
    }

    if e.server_card_valid {
        score += 25;
    } else if e.server_card_found {
        score += 8;
    }

    if e.server_cards_valid {
        score += 20;
    } else if e.server_cards_found {
        score += 8;
    }

    if e.html_marker_count > 0 {
        score += (e.html_marker_count.min(3) as u16) * 3;
    }
    if e.link_hint_count > 0 {
        score += (e.link_hint_count.min(2) as u16) * 5;
    }

    if e.endpoint_found {
        score += 10;
    }
    if e.protocol_version_found {
        score += 10;
    }
    if e.in_page_model_context_api {
        score += 22;
    }
    if e.in_page_register_tool_usage {
        score += 18;
    }
    if e.in_page_tool_schema_usage {
        score += 10;
    }
    if e.endpoint_probe_success {
        score += 20;
    }
    if e.endpoint_probe_from_manifest_success {
        score += 10;
    }

    let score = score.min(100) as u8;
    let supported = score >= 55
        || (e.manifest_valid && e.endpoint_probe_success)
        || ((e.server_card_valid || e.server_cards_valid) && e.endpoint_probe_success)
        || (e.in_page_model_context_api && e.in_page_register_tool_usage);

    let confidence = if score >= 75 {
        WebMcpConfidence::High
    } else if score >= 50 {
        WebMcpConfidence::Medium
    } else {
        WebMcpConfidence::Low
    };

    (score, confidence, supported)
}

pub fn route_appears_available(status: u16) -> bool {
    status > 0 && status != 404
}

pub fn extract_web_mcp_tools_from_html(html: &str) -> Vec<WebMcpTool> {
    let register_tool_pattern = Regex::new(r#"(?s)registerTool\s*\(\s*\{(.*?)\}\s*\)"#).ok();
    let name_pattern = Regex::new(r#"(?i)\bname\s*:\s*["']([^"']+)["']"#).ok();
    let description_pattern = Regex::new(r#"(?i)\bdescription\s*:\s*["']([^"']+)["']"#).ok();
    let (Some(register_tool_pattern), Some(name_pattern), Some(description_pattern)) =
        (register_tool_pattern, name_pattern, description_pattern)
    else {
        return Vec::new();
    };

    let mut tools = Vec::new();
    for capture in register_tool_pattern.captures_iter(html) {
        let Some(body) = capture.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let Some(name) = name_pattern
            .captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let description = description_pattern
            .captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        tools.push(WebMcpTool {
            name,
            description,
            source: Some("in-page registerTool".to_string()),
        });
    }

    tools.sort();
    tools.dedup();
    tools
}

pub fn extract_web_mcp_tools_from_json(json: &JsonValue, source: &str) -> Vec<WebMcpTool> {
    let mut tools = Vec::new();
    collect_tools_from_json(json, source, &mut tools);
    tools.sort();
    tools.dedup();
    tools
}

fn absolutize_endpoint(endpoint: &str, origin_or_base_url: &str) -> Option<String> {
    if endpoint.trim().is_empty() {
        return None;
    }
    if let Ok(url) = Url::parse(endpoint) {
        return Some(url.to_string());
    }

    let origin = normalize_origin(origin_or_base_url);
    let base = Url::parse(&origin).ok()?;
    base.join(endpoint).ok().map(|u| u.to_string())
}

fn collect_string_values(value: &JsonValue, out: &mut Vec<String>) {
    match value {
        JsonValue::String(s) => out.push(s.to_string()),
        JsonValue::Array(items) => {
            for item in items {
                collect_string_values(item, out);
            }
        }
        JsonValue::Object(map) => {
            for value in map.values() {
                collect_string_values(value, out);
            }
        }
        _ => {}
    }
}

fn extract_protocol_versions(value: &JsonValue) -> Vec<String> {
    let mut versions = Vec::new();
    collect_protocol_versions(value, &mut versions);
    versions.sort();
    versions.dedup();
    versions
}

fn collect_protocol_versions(value: &JsonValue, out: &mut Vec<String>) {
    match value {
        JsonValue::Array(items) => {
            for item in items {
                collect_protocol_versions(item, out);
            }
        }
        JsonValue::Object(map) => {
            if let Some(version) = map
                .get("protocolVersion")
                .or_else(|| map.get("protocol_version"))
                .and_then(|v| v.as_str())
            {
                out.push(version.to_string());
            }
            for nested in map.values() {
                collect_protocol_versions(nested, out);
            }
        }
        _ => {}
    }
}

fn collect_tools_from_json(value: &JsonValue, source: &str, out: &mut Vec<WebMcpTool>) {
    match value {
        JsonValue::Array(items) => {
            for item in items {
                collect_tools_from_json(item, source, out);
            }
        }
        JsonValue::Object(map) => {
            if let Some(tools) = map.get("tools").and_then(|v| v.as_array()) {
                for tool in tools {
                    let Some(tool_obj) = tool.as_object() else {
                        continue;
                    };
                    let Some(name) = tool_obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    else {
                        continue;
                    };
                    let description = tool_obj
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string);
                    out.push(WebMcpTool {
                        name: name.to_string(),
                        description,
                        source: Some(source.to_string()),
                    });
                }
            }
            for nested in map.values() {
                collect_tools_from_json(nested, source, out);
            }
        }
        _ => {}
    }
}

fn parse_protocol_date(version: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version.trim().split('-');
    let year = parts.next()?.parse::<u32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((year, month, day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_manifest_candidate() {
        let json = serde_json::json!({
            "name": "Demo MCP",
            "protocolVersion": "2025-06-18",
            "transport": { "type": "http", "endpoint": "/mcp" },
            "capabilities": { "tools": { "list": true } }
        });
        let parsed = parse_web_mcp_candidate(&json, "https://example.com", "/.well-known/mcp.json");
        assert!(parsed.valid_structure);
        assert_eq!(parsed.protocol_version.as_deref(), Some("2025-06-18"));
        assert_eq!(parsed.endpoint.as_deref(), Some("https://example.com/mcp"));
    }

    #[test]
    fn scores_high_when_multi_signal() {
        let evidence = WebMcpEvidence {
            manifest_found: true,
            manifest_valid: true,
            html_marker_count: 2,
            endpoint_found: true,
            endpoint_probe_success: true,
            protocol_version_found: true,
            ..Default::default()
        };
        let (score, confidence, supported) = score_web_mcp_evidence(&evidence);
        assert!(supported);
        assert!(score >= 75);
        assert_eq!(confidence, WebMcpConfidence::High);
    }

    #[test]
    fn detects_outdated_protocol_and_legacy_link() {
        let json = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "docs": "https://www.anthropic.com/news/agent-skills"
        });
        let findings = detect_outdated_web_mcp_details(&json);
        assert!(
            findings
                .iter()
                .any(|f| f.contains("Outdated protocolVersion"))
        );
        assert!(
            findings
                .iter()
                .any(|f| f.contains("Legacy Anthropic MCP reference"))
        );
    }

    #[test]
    fn extracts_tools_from_html_register_tool_calls() {
        let html = r#"
            <script>
                document.modelContext.registerTool({
                    name: "search_catalog",
                    description: "Search products",
                    inputSchema: { type: "object", properties: {} }
                });
            </script>
        "#;
        let tools = extract_web_mcp_tools_from_html(html);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "search_catalog");
        assert_eq!(tools[0].description.as_deref(), Some("Search products"));
    }
}
