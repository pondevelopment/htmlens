//! OpenAPI specification validation
//!
//! Validates OpenAPI/Swagger specifications for AI agent consumption.

use anyhow::{Context, Result};
use openapiv3::OpenAPI;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAPI validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiValidation {
    /// Whether the spec is valid
    pub valid: bool,
    
    /// OpenAPI version found
    pub version: Option<String>,
    
    /// API title
    pub title: Option<String>,
    
    /// API version
    pub api_version: Option<String>,
    
    /// Base server URLs
    pub servers: Vec<String>,
    
    /// List of endpoints found
    pub endpoints: Vec<EndpointInfo>,
    
    /// Issues found during validation
    pub issues: Vec<String>,
    
    /// Warnings (non-blocking)
    pub warnings: Vec<String>,
    
    /// Statistics
    pub stats: OpenApiStats,
}

/// Information about an API endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    
    /// Path (e.g., /items/{id})
    pub path: String,
    
    /// Summary description
    pub summary: Option<String>,
    
    /// Whether it has a 200 response defined
    pub has_success_response: bool,
}

/// Statistics about the OpenAPI spec
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenApiStats {
    /// Total number of paths
    pub total_paths: usize,
    
    /// Total number of operations
    pub total_operations: usize,
    
    /// Number of schemas defined
    pub total_schemas: usize,
    
    /// Whether security schemes are defined
    pub has_security: bool,
}

impl OpenApiValidation {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            valid: true,
            version: None,
            title: None,
            api_version: None,
            servers: Vec::new(),
            endpoints: Vec::new(),
            issues: Vec::new(),
            warnings: Vec::new(),
            stats: OpenApiStats::default(),
        }
    }
    
    /// Add an issue
    pub fn add_issue(&mut self, message: String) {
        self.valid = false;
        self.issues.push(message);
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
}

impl Default for OpenApiValidation {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse and validate an OpenAPI specification
pub fn validate_openapi(content: &str, is_yaml: bool) -> OpenApiValidation {
    let mut validation = OpenApiValidation::new();
    
    // Parse the spec
    let spec: OpenAPI = if is_yaml {
        match serde_yaml::from_str(content) {
            Ok(s) => s,
            Err(e) => {
                validation.add_issue(format!("Failed to parse YAML: {}", e));
                return validation;
            }
        }
    } else {
        match serde_json::from_str(content) {
            Ok(s) => s,
            Err(e) => {
                validation.add_issue(format!("Failed to parse JSON: {}", e));
                return validation;
            }
        }
    };
    
    // Extract basic info
    validation.version = Some(spec.openapi.clone());
    validation.title = Some(spec.info.title.clone());
    validation.api_version = Some(spec.info.version.clone());
    
    // Check OpenAPI version
    if !spec.openapi.starts_with("3.") {
        validation.add_warning(format!(
            "OpenAPI version {} - version 3.x is recommended",
            spec.openapi
        ));
    }
    
    // Extract server URLs
    validation.servers = spec
        .servers
        .iter()
        .map(|s| s.url.clone())
        .collect();
    
    if validation.servers.is_empty() {
        validation.add_issue("No servers defined - at least one server URL is required".to_string());
    }
    
    // Analyze paths and operations
    validation.stats.total_paths = spec.paths.paths.len();
    
    for (path, path_item_ref) in &spec.paths.paths {
        if let Some(path_item) = path_item_ref.as_item() {
            // Check each operation
            for (method, operation) in path_item.iter() {
                validation.stats.total_operations += 1;
                
                let method_str = method.to_uppercase();
                let has_200 = operation
                    .responses
                    .responses
                    .iter()
                    .any(|(code, _)| matches!(code, openapiv3::StatusCode::Code(200)));
                
                if !has_200 {
                    validation.add_warning(format!(
                        "{} {} has no 200 response defined",
                        method_str, path
                    ));
                }
                
                validation.endpoints.push(EndpointInfo {
                    method: method_str,
                    path: path.clone(),
                    summary: operation.summary.clone(),
                    has_success_response: has_200,
                });
            }
        }
    }
    
    if validation.stats.total_operations == 0 {
        validation.add_issue("No operations defined in the API".to_string());
    }
    
    // Check for schemas
    if let Some(components) = &spec.components {
        validation.stats.total_schemas = components.schemas.len();
        
        if !components.security_schemes.is_empty() {
            validation.stats.has_security = true;
        }
    }
    
    // Additional checks
    if validation.stats.total_schemas == 0 {
        validation.add_warning("No schemas defined - consider adding data models for better documentation".to_string());
    }
    
    validation
}

/// Fetch and validate OpenAPI specification from a URL
pub async fn fetch_and_validate_openapi(url: &str) -> Result<OpenApiValidation> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("htmlens-ai-readiness-checker/0.4.0")
        .build()?;
    
    let response = client.get(url).send().await
        .context("Failed to fetch OpenAPI spec")?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch OpenAPI spec: HTTP {}", response.status());
    }
    
    let content = response.text().await?;
    
    // Determine if it's YAML or JSON based on URL or content
    let is_yaml = url.ends_with(".yaml") || url.ends_with(".yml") || 
                  (!url.ends_with(".json") && content.trim_start().starts_with("openapi:"));
    
    Ok(validate_openapi(&content, is_yaml))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_openapi_json() {
        let json = r#"{
            "openapi": "3.0.1",
            "info": {
                "title": "Example API",
                "version": "1.0.0"
            },
            "servers": [
                {"url": "https://api.example.com"}
            ],
            "paths": {
                "/items": {
                    "get": {
                        "summary": "List items",
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            }
        }"#;
        
        let validation = validate_openapi(json, false);
        assert!(validation.valid, "Expected valid spec but got issues: {:?}", validation.issues);
        assert_eq!(validation.stats.total_operations, 1);
        assert_eq!(validation.endpoints.len(), 1);
    }
    
    #[test]
    fn test_missing_servers() {
        let json = r#"{
            "openapi": "3.0.1",
            "info": {
                "title": "Example API",
                "version": "1.0.0"
            },
            "paths": {}
        }"#;
        
        let validation = validate_openapi(json, false);
        assert!(!validation.valid);
        assert!(validation.issues.iter().any(|i| i.contains("No servers")));
    }
}
