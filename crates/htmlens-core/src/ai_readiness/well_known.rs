//! Checks for .well-known directory files
//!
//! The .well-known directory (RFC 8615) is a standard location for
//! site-wide metadata and configuration files.

#[cfg(feature = "ai-readiness")]
use anyhow::Result;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ai-readiness")]
use std::time::Duration;
#[cfg(feature = "ai-readiness")]
use url::Url;

/// Results from checking .well-known directory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WellKnownChecks {
    /// AI Plugin manifest check (OpenAI ChatGPT)
    pub ai_plugin: FileCheck,

    /// Model Context Protocol manifest check (Anthropic Claude)
    pub mcp: FileCheck,

    /// OpenID Connect configuration check
    pub openid_config: FileCheck,

    /// Security contact info check
    pub security_txt: FileCheck,

    /// iOS app association check
    pub apple_app_site_association: FileCheck,

    /// Android app association check
    pub assetlinks: FileCheck,
}

/// Status of a specific .well-known file check
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileCheck {
    /// The file path checked
    pub path: String,

    /// HTTP status code received
    pub status_code: Option<u16>,

    /// Whether the file was found
    pub found: bool,

    /// Whether the content is valid
    pub valid: bool,

    /// Error message if check failed
    pub error: Option<String>,

    /// File content if successfully retrieved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl FileCheck {
    /// Create a new file check for a given path
    pub fn new(path: String) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }
}

/// Check all relevant .well-known files for a domain
#[cfg(feature = "ai-readiness")]
pub async fn check_well_known_files(base_url: &str) -> Result<WellKnownChecks> {
    let normalized_base = normalize_base_url(base_url);
    let base_url = normalized_base.trim_end_matches('/');

    let mut checks = WellKnownChecks {
        ai_plugin: FileCheck::new("/.well-known/ai-plugin.json".to_string()),
        mcp: FileCheck::new("/.well-known/mcp.json".to_string()),
        openid_config: FileCheck::new("/.well-known/openid-configuration".to_string()),
        security_txt: FileCheck::new("/.well-known/security.txt".to_string()),
        apple_app_site_association: FileCheck::new(
            "/.well-known/apple-app-site-association".to_string(),
        ),
        assetlinks: FileCheck::new("/.well-known/assetlinks.json".to_string()),
    };

    // Check each file
    checks.ai_plugin = check_file(base_url, &checks.ai_plugin.path, FileType::Json).await;
    checks.mcp = check_file(base_url, &checks.mcp.path, FileType::Json).await;
    checks.openid_config = check_file(base_url, &checks.openid_config.path, FileType::Json).await;
    checks.security_txt = check_file(base_url, &checks.security_txt.path, FileType::Text).await;
    checks.apple_app_site_association = check_file(
        base_url,
        &checks.apple_app_site_association.path,
        FileType::Json,
    )
    .await;
    checks.assetlinks = check_file(base_url, &checks.assetlinks.path, FileType::Json).await;

    Ok(checks)
}

/// Normalize an input URL to its origin (scheme + host [+ port]).
#[cfg(feature = "ai-readiness")]
fn normalize_base_url(base_url: &str) -> String {
    if let Ok(mut parsed) = Url::parse(base_url) {
        parsed.set_path("");
        parsed.set_query(None);
        parsed.set_fragment(None);
        parsed.to_string()
    } else {
        base_url.to_string()
    }
}

/// Type of file expected
#[cfg(feature = "ai-readiness")]
#[derive(Debug, Clone, Copy)]
enum FileType {
    Json,
    Text,
}

/// Check a specific .well-known file
#[cfg(feature = "ai-readiness")]
async fn check_file(base_url: &str, path: &str, file_type: FileType) -> FileCheck {
    let url = format!("{}{}", base_url, path);
    let mut check = FileCheck::new(path.to_string());

    // Create HTTP client with timeout
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("htmlens-ai-readiness-checker/0.4.2")
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            check.error = Some(format!("Failed to create HTTP client: {}", e));
            return check;
        }
    };

    // Make request
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            check.error = Some(format!("Request failed: {}", e));
            return check;
        }
    };

    check.status_code = Some(response.status().as_u16());
    check.found = response.status().is_success();

    if !check.found {
        if response.status().as_u16() == 404 {
            check.error = Some("File not found (404)".to_string());
        } else {
            check.error = Some(format!("HTTP error: {}", response.status()));
        }
        return check;
    }

    // Get content
    let content = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            check.error = Some(format!("Failed to read response: {}", e));
            return check;
        }
    };

    // Validate content based on type
    check.valid = match file_type {
        FileType::Json => validate_json(&content),
        FileType::Text => validate_text(&content),
    };

    if !check.valid {
        check.error = Some(match file_type {
            FileType::Json => "Invalid JSON format".to_string(),
            FileType::Text => "Invalid or empty content".to_string(),
        });
    }

    check.content = Some(content);
    check
}

/// Validate JSON content
#[cfg(feature = "ai-readiness")]
fn validate_json(content: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(content).is_ok()
}

/// Validate text content (just check it's not empty)
#[cfg(feature = "ai-readiness")]
fn validate_text(content: &str) -> bool {
    !content.trim().is_empty()
}

#[cfg(all(test, feature = "ai-readiness"))]
mod tests {
    use super::*;

    #[test]
    fn test_validate_json() {
        assert!(validate_json(r#"{"key": "value"}"#));
        assert!(validate_json(r#"[]"#));
        assert!(!validate_json("not json"));
        assert!(!validate_json(""));
    }

    #[test]
    fn test_validate_text() {
        assert!(validate_text("Contact: security@example.com"));
        assert!(!validate_text(""));
        assert!(!validate_text("   "));
    }

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(
            normalize_base_url("https://example.com/path/page.html"),
            "https://example.com/"
        );
        assert_eq!(
            normalize_base_url("https://example.com"),
            "https://example.com/"
        );
        assert_eq!(
            normalize_base_url("https://example.com:8443/foo"),
            "https://example.com:8443/"
        );
        assert_eq!(normalize_base_url("not a url"), "not a url");
    }
}
