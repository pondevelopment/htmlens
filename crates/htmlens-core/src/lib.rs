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
//! use htmlens_core::graph::KnowledgeGraph;
//! 
//! # async fn example() -> anyhow::Result<()> {
//! let html = r#"
//!     <script type="application/ld+json">
//!     {"@context": "https://schema.org", "@type": "Product", "name": "Example"}
//!     </script>
//! "#;
//! 
//! let blocks = parser::extract_json_ld_blocks(html)?;
//! let json_ld = parser::combine_json_ld_blocks(blocks)?;
//! # Ok(())
//! # }
//! ```

pub mod parser;
pub mod types;

#[cfg(feature = "full-expansion")]
pub mod graph;

// Re-export commonly used types
pub use types::{GraphNode, GraphEdge, KnowledgeGraph};

pub use parser::{
    extract_json_ld_blocks,
    combine_json_ld_blocks,
    sanitize_html,
    html_to_markdown,
};

#[cfg(feature = "full-expansion")]
pub use parser::fetch_html;

#[cfg(feature = "full-expansion")]
pub use graph::{
    GraphBuilder,
    expand_json_ld,
};
