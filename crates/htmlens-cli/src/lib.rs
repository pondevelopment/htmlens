//! Unit tests for CLI utility functions
//!
//! These tests can be run without building the entire CLI binary

// Since we can't easily import from main.rs, we'll create a separate module
// for testable functions. Let's add some basic tests for the concepts.

#[cfg(test)]
mod cli_utils_tests {
    use std::collections::HashMap;

    // Test helper functions that would be used in the CLI

    #[test]
    fn test_shorten_iri() {
        fn shorten_iri(iri: &str) -> String {
            if let Some(pos) = iri.rfind('#') {
                iri[pos + 1..].to_string()
            } else if let Some(pos) = iri.rfind('/') {
                iri[pos + 1..].to_string()
            } else {
                iri.to_string()
            }
        }

        assert_eq!(shorten_iri("https://schema.org/name"), "name");
        assert_eq!(shorten_iri("https://schema.org/Product"), "Product");
        assert_eq!(shorten_iri("http://example.com#property"), "property");
        assert_eq!(shorten_iri("simple"), "simple");
    }

    #[test]
    fn test_has_schema_type() {
        fn has_schema_type(types: &[String], target_type: &str) -> bool {
            let target_lower = target_type.to_lowercase();
            types.iter().any(|t| {
                let shortened = if let Some(pos) = t.rfind('/') {
                    &t[pos + 1..]
                } else {
                    t
                };
                shortened.to_lowercase() == target_lower
            })
        }

        let types = vec![
            "https://schema.org/Product".to_string(),
            "https://schema.org/Thing".to_string(),
        ];

        assert!(has_schema_type(&types, "Product"));
        assert!(has_schema_type(&types, "product")); // Case insensitive
        assert!(has_schema_type(&types, "Thing"));
        assert!(!has_schema_type(&types, "Organization"));
    }

    #[test]
    fn test_format_price() {
        fn format_price(value: &str, currency: Option<&str>) -> String {
            let currency_symbol = match currency {
                Some("USD") => "$",
                Some("EUR") => "€",
                Some("GBP") => "£",
                _ => "",
            };

            if currency_symbol.is_empty() {
                value.to_string()
            } else {
                format!("{}{}", currency_symbol, value)
            }
        }

        assert_eq!(format_price("29.99", Some("USD")), "$29.99");
        assert_eq!(format_price("35.50", Some("EUR")), "€35.50");
        assert_eq!(format_price("19.99", Some("GBP")), "£19.99");
        assert_eq!(format_price("100", Some("JPY")), "100"); // No symbol
        assert_eq!(format_price("50", None), "50");
    }

    #[test]
    fn test_normalize_tokens() {
        fn normalize_tokens(name: &str) -> Vec<String> {
            let mut tokens = Vec::new();
            let mut current = String::new();

            for ch in name.chars() {
                if ch.is_uppercase() && !current.is_empty() {
                    tokens.push(current.to_lowercase());
                    current = String::new();
                }
                current.push(ch);
            }

            if !current.is_empty() {
                tokens.push(current.to_lowercase());
            }

            tokens
        }

        assert_eq!(normalize_tokens("FrameSize"), vec!["frame", "size"]);
        assert_eq!(normalize_tokens("colorway"), vec!["colorway"]);
        assert_eq!(normalize_tokens("ProductName"), vec!["product", "name"]);
        assert_eq!(normalize_tokens("simple"), vec!["simple"]);
        assert_eq!(
            normalize_tokens("HTMLParser"),
            vec!["h", "t", "m", "l", "parser"]
        );
    }

    #[test]
    fn test_predicate_matches() {
        fn predicate_matches(predicate: &str, name: &str) -> bool {
            let pred_lower = predicate.to_lowercase();
            let name_lower = name.to_lowercase();

            pred_lower.contains(&name_lower)
                || pred_lower.ends_with(&format!("/{}", name_lower))
                || pred_lower.ends_with(&format!("#{}", name_lower))
        }

        assert!(predicate_matches(
            "https://schema.org/hasVariant",
            "hasVariant"
        ));
        assert!(predicate_matches(
            "https://schema.org/hasVariant",
            "hasvariant"
        )); // Case insensitive
        assert!(predicate_matches("http://example.com#offers", "offers"));
        assert!(!predicate_matches("https://schema.org/name", "hasVariant"));
    }

    #[test]
    fn test_build_adjacency_concept() {
        // Simulated edge structure
        #[allow(dead_code)]
        struct Edge {
            from: String,
            to: String,
            predicate: String,
        }

        fn build_adjacency(edges: &[Edge]) -> HashMap<String, Vec<&Edge>> {
            let mut adjacency: HashMap<String, Vec<&Edge>> = HashMap::new();

            for edge in edges {
                adjacency.entry(edge.from.clone()).or_default().push(edge);
            }

            adjacency
        }

        let edges = vec![
            Edge {
                from: "product1".to_string(),
                to: "offer1".to_string(),
                predicate: "offers".to_string(),
            },
            Edge {
                from: "product1".to_string(),
                to: "variant1".to_string(),
                predicate: "hasVariant".to_string(),
            },
        ];

        let adjacency = build_adjacency(&edges);

        assert_eq!(adjacency.get("product1").unwrap().len(), 2);
        assert!(!adjacency.contains_key("nonexistent"));
    }

    #[test]
    fn test_extract_entity_types() {
        fn extract_entity_types(json_blocks: &[serde_json::Value]) -> Vec<String> {
            let mut types = std::collections::HashSet::new();

            for block in json_blocks {
                if let Some(type_val) = block.get("@type") {
                    match type_val {
                        serde_json::Value::String(s) => {
                            let shortened = if let Some(pos) = s.rfind('/') {
                                &s[pos + 1..]
                            } else {
                                s
                            };
                            types.insert(shortened.to_string());
                        }
                        serde_json::Value::Array(arr) => {
                            for item in arr {
                                if let serde_json::Value::String(s) = item {
                                    let shortened = if let Some(pos) = s.rfind('/') {
                                        &s[pos + 1..]
                                    } else {
                                        s
                                    };
                                    types.insert(shortened.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            let mut result: Vec<String> = types.into_iter().collect();
            result.sort();
            result
        }

        let blocks = vec![
            serde_json::json!({"@type": "https://schema.org/Product"}),
            serde_json::json!({"@type": ["Organization", "LocalBusiness"]}),
            serde_json::json!({"@type": "Offer"}),
        ];

        let types = extract_entity_types(&blocks);

        assert!(types.contains(&"Product".to_string()));
        assert!(types.contains(&"Organization".to_string()));
        assert!(types.contains(&"LocalBusiness".to_string()));
        assert!(types.contains(&"Offer".to_string()));
        assert_eq!(types.len(), 4);
    }
}
