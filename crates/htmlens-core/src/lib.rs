//! # htmlens-core
//!
//! Core library for extracting semantic knowledge graphs from HTML pages.
//!
//! This library provides:
//! - HTML parsing and JSON-LD extraction
//! - Knowledge graph construction from JSON-LD
//! - Schema.org entity type detection
//!
//! ## Features
//!
//! - `default`: Basic HTML parsing and JSON-LD extraction (no expansion)
//! - `full-expansion`: Complete JSON-LD expansion with remote context resolution
//!
//! ## Example
//!
//! ```no_run
//! use htmlens_core::parser;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let html = r#"
//!     <script type="application/ld+json">
//!     {"@context": "https://schema.org", "@type": "Product", "name": "Example"}
//!     </script>
//! "#;
//!
//! let blocks = parser::extract_json_ld_blocks(html)?;
//! let json_ld = parser::combine_json_ld_blocks(&blocks)?;
//! # Ok(())
//! # }
//! ```

pub mod parser;
pub mod types;

#[cfg(feature = "full-expansion")]
pub mod graph;

#[cfg(any(feature = "ai-readiness", feature = "ai-readiness-parser"))]
pub mod ai_readiness;

// Re-export commonly used types
pub use types::{GraphEdge, GraphNode, KnowledgeGraph};

pub use parser::{combine_json_ld_blocks, extract_json_ld_blocks, html_to_markdown, sanitize_html};

#[cfg(feature = "full-expansion")]
pub use parser::fetch_html;

#[cfg(feature = "full-expansion")]
pub use graph::{GraphBuilder, expand_json_ld};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_types_creation() {
        let node = GraphNode {
            id: "test-node".to_string(),
            types: vec!["Product".to_string()],
            properties: HashMap::new(),
        };

        assert_eq!(node.id, "test-node");
        assert_eq!(node.types, vec!["Product"]);
        assert!(node.properties.is_empty());
    }

    #[test]
    fn test_edge_creation() {
        let edge = GraphEdge {
            from: "node1".to_string(),
            to: "node2".to_string(),
            predicate: "hasVariant".to_string(),
        };

        assert_eq!(edge.from, "node1");
        assert_eq!(edge.to, "node2");
        assert_eq!(edge.predicate, "hasVariant");
    }

    #[test]
    fn test_knowledge_graph_creation() {
        let graph = KnowledgeGraph {
            nodes: vec![],
            edges: vec![],
        };

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_extract_json_ld_from_html() {
        let html = r#"
            <html>
                <head>
                    <script type="application/ld+json">
                    {"@context": "https://schema.org", "@type": "Product", "name": "Test Product"}
                    </script>
                </head>
                <body>Some content</body>
            </html>
        "#;

        let blocks = extract_json_ld_blocks(html).unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("Test Product"));
    }

    #[test]
    fn test_extract_multiple_json_ld_blocks() {
        let html = r#"
            <html>
                <head>
                    <script type="application/ld+json">
                    {"@type": "Product", "name": "Product 1"}
                    </script>
                    <script type="application/ld+json">
                    {"@type": "Organization", "name": "Company"}
                    </script>
                </head>
            </html>
        "#;

        let blocks = extract_json_ld_blocks(html).unwrap();
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_combine_json_ld_blocks() {
        let blocks = vec![
            r#"{"@context": "https://schema.org", "@type": "Product", "name": "Product 1"}"#
                .to_string(),
            r#"{"@type": "Organization", "name": "Company"}"#.to_string(),
        ];

        let combined = combine_json_ld_blocks(&blocks).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&combined).unwrap();

        assert_eq!(parsed["@context"], "https://schema.org");
        assert!(parsed["@graph"].is_array());
        assert_eq!(parsed["@graph"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_sanitize_html() {
        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <h1>Main Content</h1>
                    <script>alert('xss')</script>
                    <style>body { color: red; }</style>
                    <p>Safe content</p>
                </body>
            </html>
        "#;

        let sanitized = sanitize_html(html);

        // Should keep safe content
        assert!(sanitized.contains("Main Content"));
        assert!(sanitized.contains("Safe content"));

        // Should remove dangerous content
        assert!(!sanitized.contains("<script"));
        assert!(!sanitized.contains("<style"));
        assert!(!sanitized.contains("alert"));
    }

    #[test]
    fn test_html_to_markdown() {
        let html = r#"
            <h1>Main Title</h1>
            <p>This is a paragraph with <strong>bold</strong> text.</p>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
        "#;

        let markdown = html_to_markdown(html);

        // html2md uses alternative heading format for h1 and * for lists
        assert!(markdown.contains("Main Title\n=========="));
        assert!(markdown.contains("**bold**"));
        assert!(markdown.contains("* Item 1"));
        assert!(markdown.contains("* Item 2"));
    }
}
