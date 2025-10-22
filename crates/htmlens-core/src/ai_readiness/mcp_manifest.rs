//! Model Context Protocol (MCP) manifest validation
//!
//! MCP is an emerging standard by Anthropic for AI agent integration.
//! It provides a structured way to expose tools, resources, and prompts.
//!
//! Specification: https://modelcontextprotocol.io/

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpManifest {
    /// Schema version (e.g., "1.0")
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    
    /// Protocol version (e.g., "2025-06-18")
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    
    /// Service name
    pub name: String,
    
    /// Service description
    pub description: String,
    
    /// Service version
    pub version: String,
    
    /// Supported protocol versions
    #[serde(rename = "supportedProtocolVersions", skip_serializing_if = "Option::is_none")]
    pub supported_protocol_versions: Option<Vec<String>>,
    
    /// Capabilities offered
    pub capabilities: McpCapabilities,
    
    /// Transport configuration
    pub transport: McpTransport,
    
    /// Available tools
    #[serde(default)]
    pub tools: Vec<McpTool>,
    
    /// Available resources
    #[serde(default)]
    pub resources: Vec<McpResource>,
    
    /// Available prompts
    #[serde(default)]
    pub prompts: Vec<McpPrompt>,
    
    /// Health check configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<McpHealthConfig>,
}

/// MCP capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    /// Tools capability
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
    
    /// Resources capability
    #[serde(default)]
    pub resources: Option<ResourcesCapability>,
    
    /// Prompts capability
    #[serde(default)]
    pub prompts: Option<PromptsCapability>,
    
    /// Events capability
    #[serde(default)]
    pub events: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(default)]
    pub list: Option<bool>,
    #[serde(default)]
    pub call: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(default)]
    pub list: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(default)]
    pub list: Option<bool>,
}

/// MCP transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTransport {
    /// Transport type: "http" or "sse"
    #[serde(rename = "type")]
    pub transport_type: String,
    
    /// Endpoint URL
    pub endpoint: String,
    
    /// Authorization type
    pub authorization: String,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
    
    /// Input schema (JSON Schema)
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Resource URI
    pub uri: String,
    
    /// Resource name
    pub name: String,
    
    /// Resource description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// MIME type
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// MCP prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    /// Prompt name
    pub name: String,
    
    /// Prompt description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Prompt arguments
    #[serde(default)]
    pub arguments: Vec<McpPromptArgument>,
}

/// MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    /// Argument name
    pub name: String,
    
    /// Argument description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Whether argument is required
    #[serde(default)]
    pub required: bool,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHealthConfig {
    /// Health check endpoint
    pub endpoint: String,
}

/// Validation result for MCP manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpManifestValidation {
    /// Whether the manifest is valid
    pub valid: bool,
    
    /// Service name
    pub name: Option<String>,
    
    /// Service version
    pub version: Option<String>,
    
    /// Protocol version
    pub protocol_version: Option<String>,
    
    /// Transport type
    pub transport_type: Option<String>,
    
    /// Endpoint URL
    pub endpoint: Option<String>,
    
    /// Authorization type
    pub authorization: Option<String>,
    
    /// Number of tools
    pub tool_count: usize,
    
    /// Number of resources
    pub resource_count: usize,
    
    /// Number of prompts
    pub prompt_count: usize,
    
    /// Whether tools capability is supported
    pub has_tools_capability: bool,
    
    /// Whether resources capability is supported
    pub has_resources_capability: bool,
    
    /// Whether prompts capability is supported
    pub has_prompts_capability: bool,
    
    /// Whether events capability is supported
    pub has_events_capability: bool,
    
    /// Health check endpoint
    pub health_endpoint: Option<String>,
    
    /// Validation issues
    pub issues: Vec<String>,
}

/// Parse and validate an MCP manifest
pub fn validate_manifest(json_str: &str) -> Result<McpManifestValidation> {
    let manifest: McpManifest = serde_json::from_str(json_str)?;
    
    let mut validation = McpManifestValidation {
        valid: true,
        name: Some(manifest.name.clone()),
        version: Some(manifest.version.clone()),
        protocol_version: Some(manifest.protocol_version.clone()),
        transport_type: Some(manifest.transport.transport_type.clone()),
        endpoint: Some(manifest.transport.endpoint.clone()),
        authorization: Some(manifest.transport.authorization.clone()),
        tool_count: manifest.tools.len(),
        resource_count: manifest.resources.len(),
        prompt_count: manifest.prompts.len(),
        has_tools_capability: manifest.capabilities.tools.is_some(),
        has_resources_capability: manifest.capabilities.resources.is_some(),
        has_prompts_capability: manifest.capabilities.prompts.is_some(),
        has_events_capability: manifest.capabilities.events.is_some(),
        health_endpoint: manifest.health.as_ref().map(|h| h.endpoint.clone()),
        issues: Vec::new(),
    };
    
    // Validate required fields
    if manifest.name.is_empty() {
        validation.valid = false;
        validation.issues.push("Missing or empty 'name' field".to_string());
    }
    
    if manifest.schema_version.is_empty() {
        validation.valid = false;
        validation.issues.push("Missing or empty 'schemaVersion' field".to_string());
    }
    
    if manifest.protocol_version.is_empty() {
        validation.valid = false;
        validation.issues.push("Missing or empty 'protocolVersion' field".to_string());
    }
    
    // Validate transport
    if manifest.transport.transport_type.is_empty() {
        validation.valid = false;
        validation.issues.push("Missing or empty transport type".to_string());
    } else if manifest.transport.transport_type != "http" && manifest.transport.transport_type != "sse" {
        validation.issues.push(format!(
            "Unknown transport type '{}' (expected 'http' or 'sse')",
            manifest.transport.transport_type
        ));
    }
    
    if manifest.transport.endpoint.is_empty() {
        validation.valid = false;
        validation.issues.push("Missing or empty transport endpoint".to_string());
    } else if let Err(e) = url::Url::parse(&manifest.transport.endpoint) {
        validation.valid = false;
        validation.issues.push(format!("Invalid endpoint URL: {}", e));
    }
    
    // Validate tools
    for tool in &manifest.tools {
        if tool.name.is_empty() {
            validation.issues.push("Tool with empty name found".to_string());
        }
        
        if tool.description.is_empty() {
            validation.issues.push(format!("Tool '{}' missing description", tool.name));
        }
        
        // Check if input schema is valid JSON Schema
        if !tool.input_schema.is_object() {
            validation.issues.push(format!(
                "Tool '{}' has invalid input schema (must be object)",
                tool.name
            ));
        }
    }
    
    // Validate capabilities match actual content
    if manifest.tool_count > 0 && !manifest.has_tools_capability {
        validation.issues.push("Tools defined but tools capability not declared".to_string());
    }
    
    if manifest.resource_count > 0 && !manifest.has_resources_capability {
        validation.issues.push("Resources defined but resources capability not declared".to_string());
    }
    
    if manifest.prompt_count > 0 && !manifest.has_prompts_capability {
        validation.issues.push("Prompts defined but prompts capability not declared".to_string());
    }
    
    Ok(validation)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_manifest() {
        let json = r#"{
            "schemaVersion": "1.0",
            "protocolVersion": "2025-06-18",
            "name": "Test Service",
            "description": "A test MCP service",
            "version": "1.0.0",
            "capabilities": {
                "tools": {"list": true, "call": true},
                "resources": {"list": true},
                "prompts": {"list": true},
                "events": {}
            },
            "transport": {
                "type": "http",
                "endpoint": "https://example.com/mcp",
                "authorization": "none"
            },
            "tools": [
                {
                    "name": "test_tool",
                    "description": "A test tool",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {"type": "string"}
                        },
                        "required": ["query"]
                    }
                }
            ],
            "resources": [],
            "prompts": []
        }"#;
        
        let result = validate_manifest(json);
        assert!(result.is_ok());
        
        let validation = result.unwrap();
        assert!(validation.valid);
        assert_eq!(validation.name, Some("Test Service".to_string()));
        assert_eq!(validation.tool_count, 1);
        assert!(validation.has_tools_capability);
    }
    
    #[test]
    fn test_invalid_manifest() {
        let json = r#"{
            "schemaVersion": "",
            "protocolVersion": "2025-06-18",
            "name": "",
            "description": "Invalid",
            "version": "1.0.0",
            "capabilities": {},
            "transport": {
                "type": "invalid",
                "endpoint": "not-a-url",
                "authorization": "none"
            }
        }"#;
        
        let result = validate_manifest(json);
        assert!(result.is_ok());
        
        let validation = result.unwrap();
        assert!(!validation.valid);
        assert!(!validation.issues.is_empty());
    }
}
