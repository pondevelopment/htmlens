//! Common types used across htmlens

use serde::Serialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// A node in the knowledge graph
#[derive(Debug, Serialize, Clone)]
pub struct GraphNode {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub types: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, JsonValue>,
}

/// An edge connecting two nodes in the knowledge graph
#[derive(Debug, Serialize, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub predicate: String,
}

/// A complete knowledge graph
#[derive(Debug, Serialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl GraphNode {
    pub fn new(id: String) -> Self {
        Self {
            id,
            types: Vec::new(),
            properties: HashMap::new(),
        }
    }
}
