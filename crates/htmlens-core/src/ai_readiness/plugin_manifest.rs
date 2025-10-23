//! AI Plugin Manifest validation
//!
//! Validates ChatGPT plugin manifests according to the specification.
//! See: https://platform.openai.com/docs/plugins

#[cfg(feature = "ai-readiness")]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use url::Url;

/// AI Plugin Manifest structure (schema version v1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPluginManifest {
    /// Schema version (currently "v1")
    pub schema_version: String,
    
    /// Human-readable plugin name
    pub name_for_human: String,
    
    /// Machine-readable plugin name (no spaces)
    pub name_for_model: String,
    
    /// Human-facing description
    pub description_for_human: String,
    
    /// Model-facing description (up to ~8000 chars)
    pub description_for_model: String,
    
    /// Authentication configuration
    pub auth: AuthConfig,
    
    /// API specification
    pub api: ApiConfig,
    
    /// Logo URL
    pub logo_url: String,
    
    /// Contact email
    pub contact_email: String,
    
    /// Legal info URL
    pub legal_info_url: String,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type: "none", "user_http", or "service_http"
    #[serde(rename = "type")]
    pub auth_type: String,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API type (currently always "openapi")
    #[serde(rename = "type")]
    pub api_type: String,
    
    /// URL to OpenAPI specification
    pub url: String,
    
    /// Whether user authentication is included
    #[serde(default)]
    pub is_user_authenticated: bool,
}

/// Validation result for AI Plugin Manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestValidation {
    /// Whether the manifest is valid
    pub valid: bool,
    
    /// Issues found during validation
    pub issues: Vec<ValidationIssue>,
    
    /// Warnings (non-blocking issues)
    pub warnings: Vec<String>,
    
    /// The parsed manifest (if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<AiPluginManifest>,
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Field that has the issue
    pub field: String,
    
    /// Description of the issue
    pub message: String,
}

impl ManifestValidation {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            warnings: Vec::new(),
            manifest: None,
        }
    }
    
    /// Add an issue
    pub fn add_issue(&mut self, field: &str, message: &str) {
        self.valid = false;
        self.issues.push(ValidationIssue {
            field: field.to_string(),
            message: message.to_string(),
        });
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }
}

impl Default for ManifestValidation {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse and validate an AI Plugin Manifest
pub fn validate_manifest(json_content: &str) -> ManifestValidation {
    let mut validation = ManifestValidation::new();
    
    // Parse JSON
    let manifest: AiPluginManifest = match serde_json::from_str(json_content) {
        Ok(m) => m,
        Err(e) => {
            validation.add_issue("json", &format!("Invalid JSON: {}", e));
            return validation;
        }
    };
    
    // Validate schema version
    if manifest.schema_version != "v1" {
        validation.add_warning(&format!(
            "Schema version '{}' may not be supported (expected 'v1')",
            manifest.schema_version
        ));
    }
    
    // Validate name_for_model (no spaces allowed)
    if manifest.name_for_model.contains(' ') {
        validation.add_issue(
            "name_for_model",
            "Must not contain spaces (used as namespace identifier)",
        );
    }
    
    if manifest.name_for_model.is_empty() {
        validation.add_issue("name_for_model", "Cannot be empty");
    }
    
    // Validate names are not too long
    if manifest.name_for_human.len() > 50 {
        validation.add_warning("name_for_human is longer than 50 characters");
    }
    
    if manifest.name_for_model.len() > 50 {
        validation.add_warning("name_for_model is longer than 50 characters");
    }
    
    // Validate descriptions
    if manifest.description_for_human.is_empty() {
        validation.add_issue("description_for_human", "Cannot be empty");
    }
    
    if manifest.description_for_model.is_empty() {
        validation.add_issue("description_for_model", "Cannot be empty");
    }
    
    if manifest.description_for_model.len() > 8000 {
        validation.add_issue(
            "description_for_model",
            "Exceeds maximum length of 8000 characters",
        );
    }
    
    if manifest.description_for_model.len() < 100 {
        validation.add_warning(
            "description_for_model is short (< 100 chars) - consider adding more detail for better AI understanding"
        );
    }
    
    // Validate auth type
    let valid_auth_types = ["none", "user_http", "service_http"];
    if !valid_auth_types.contains(&manifest.auth.auth_type.as_str()) {
        validation.add_issue(
            "auth.type",
            &format!("Invalid auth type '{}' (must be: none, user_http, or service_http)", manifest.auth.auth_type),
        );
    }
    
    // Validate API type
    if manifest.api.api_type != "openapi" {
        validation.add_issue(
            "api.type",
            &format!("Invalid API type '{}' (currently only 'openapi' is supported)", manifest.api.api_type),
        );
    }
    
    // Validate URLs
    if let Err(e) = Url::parse(&manifest.api.url) {
        validation.add_issue("api.url", &format!("Invalid URL: {}", e));
    }
    
    if let Err(e) = Url::parse(&manifest.logo_url) {
        validation.add_issue("logo_url", &format!("Invalid URL: {}", e));
    }
    
    if let Err(e) = Url::parse(&manifest.legal_info_url) {
        validation.add_issue("legal_info_url", &format!("Invalid URL: {}", e));
    }
    
    // Validate email (basic check)
    if !manifest.contact_email.contains('@') {
        validation.add_issue("contact_email", "Invalid email format");
    }
    
    // Store manifest if valid
    if validation.valid {
        validation.manifest = Some(manifest);
    }
    
    validation
}

/// Fetch and validate AI plugin manifest from a URL
#[cfg(feature = "ai-readiness")]
pub async fn fetch_and_validate_manifest(base_url: &str) -> Result<ManifestValidation> {
    let url = format!("{}/.well-known/ai-plugin.json", base_url.trim_end_matches('/'));
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("htmlens-ai-readiness-checker/0.4.2")
        .build()?;
    
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch manifest: HTTP {}", response.status());
    }
    
    let content = response.text().await?;
    Ok(validate_manifest(&content))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_manifest() {
        let json = r#"{
            "schema_version": "v1",
            "name_for_human": "TODO Plugin",
            "name_for_model": "todo",
            "description_for_human": "Plugin for managing a TODO list.",
            "description_for_model": "Plugin for managing a TODO list. You can add, remove and view your TODOs with this plugin.",
            "auth": { "type": "none" },
            "api": {
                "type": "openapi",
                "url": "https://example.com/openapi.yaml",
                "is_user_authenticated": false
            },
            "logo_url": "https://example.com/logo.png",
            "contact_email": "support@example.com",
            "legal_info_url": "https://example.com/legal"
        }"#;
        
        let validation = validate_manifest(json);
        assert!(validation.valid, "Expected valid manifest but got issues: {:?}", validation.issues);
        assert!(validation.manifest.is_some());
    }
    
    #[test]
    fn test_invalid_name_with_spaces() {
        let json = r#"{
            "schema_version": "v1",
            "name_for_human": "TODO Plugin",
            "name_for_model": "todo plugin",
            "description_for_human": "Plugin for managing a TODO list.",
            "description_for_model": "Plugin for managing a TODO list. You can add, remove and view your TODOs.",
            "auth": { "type": "none" },
            "api": {
                "type": "openapi",
                "url": "https://example.com/openapi.yaml",
                "is_user_authenticated": false
            },
            "logo_url": "https://example.com/logo.png",
            "contact_email": "support@example.com",
            "legal_info_url": "https://example.com/legal"
        }"#;
        
        let validation = validate_manifest(json);
        assert!(!validation.valid);
        assert!(validation.issues.iter().any(|i| i.field == "name_for_model"));
    }
    
    #[test]
    fn test_invalid_urls() {
        let json = r#"{
            "schema_version": "v1",
            "name_for_human": "TODO Plugin",
            "name_for_model": "todo",
            "description_for_human": "Plugin for managing a TODO list.",
            "description_for_model": "Plugin for managing a TODO list. You can add, remove and view your TODOs.",
            "auth": { "type": "none" },
            "api": {
                "type": "openapi",
                "url": "not-a-valid-url",
                "is_user_authenticated": false
            },
            "logo_url": "https://example.com/logo.png",
            "contact_email": "support@example.com",
            "legal_info_url": "https://example.com/legal"
        }"#;
        
        let validation = validate_manifest(json);
        assert!(!validation.valid);
        assert!(validation.issues.iter().any(|i| i.field == "api.url"));
    }
}
