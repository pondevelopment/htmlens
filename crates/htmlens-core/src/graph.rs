//! JSON-LD expansion and knowledge graph building

use anyhow::{Result, anyhow};
use iref::IriBuf;
use json_ld::object::Literal;
use json_ld::syntax::{Parse, Value};
use json_ld::{JsonLdProcessor, RemoteDocument, ReqwestLoader};
use json_syntax::Value as SyntaxValue;
use serde_json::{Map, Value as JsonValue};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use uuid::Uuid;

// Re-export types from the types module
pub use crate::types::{GraphEdge, GraphNode, KnowledgeGraph};

/// Expand a JSON-LD block into an expanded document
pub async fn expand_json_ld(
    base_url: &str,
    raw_json_ld: &str,
    loader: &mut ReqwestLoader,
) -> Result<json_ld::ExpandedDocument> {
    let (value, _) = Value::parse_str(raw_json_ld)
        .map_err(|err| anyhow!("failed to parse JSON-LD block: {err}"))?;

    let base_iri = IriBuf::new(base_url.to_string())
        .map_err(|_| anyhow!("invalid URL provided: {base_url}"))?;

    let remote = RemoteDocument::new(
        Some(base_iri),
        Some("application/ld+json".parse().unwrap()),
        value,
    );

    remote
        .expand(loader)
        .await
        .map_err(|err| anyhow!("JSON-LD expansion failed: {err}"))
}

/// Build a knowledge graph from expanded JSON-LD documents
pub struct GraphBuilder {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
    processing: HashSet<String>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            processing: HashSet::new(),
        }
    }

    pub fn ingest_document(&mut self, doc: &json_ld::ExpandedDocument) {
        for object in doc.iter() {
            self.process_indexed_object(object);
        }
    }

    fn process_indexed_object(
        &mut self,
        object: &json_ld::IndexedObject<iref::IriBuf, rdf_types::BlankIdBuf>,
    ) -> Option<String> {
        match object.as_ref() {
            json_ld::Object::Node(node) => Some(self.process_node(node)),
            json_ld::Object::List(list) => {
                for item in list.iter() {
                    self.process_indexed_object(item);
                }
                None
            }
            json_ld::Object::Value(_) => None,
        }
    }

    fn process_node(
        &mut self,
        node: &json_ld::Node<iref::IriBuf, rdf_types::BlankIdBuf>,
    ) -> String {
        let node_id = node_identifier(node.id.as_ref());
        self.nodes
            .entry(node_id.clone())
            .or_insert_with(|| GraphNode::new(node_id.clone()));

        if !self.processing.insert(node_id.clone()) {
            return node_id;
        }

        if let Some(types) = &node.types {
            for ty in types {
                let ty_str = id_to_string(ty);
                let entry = self
                    .nodes
                    .get_mut(&node_id)
                    .expect("node should exist before types update");
                if !entry.types.contains(&ty_str) {
                    entry.types.push(ty_str);
                }
            }
        }

        if let Some(graph) = &node.graph {
            for object in graph.iter() {
                if let Some(target_id) = self.process_indexed_object(object) {
                    self.edges.push(GraphEdge {
                        from: node_id.clone(),
                        to: target_id,
                        predicate: "@graph".to_string(),
                    });
                }
            }
        }

        if let Some(included) = &node.included {
            for indexed_node in included.iter() {
                let target_id = self.process_node(indexed_node.as_ref());
                self.edges.push(GraphEdge {
                    from: node_id.clone(),
                    to: target_id,
                    predicate: "@included".to_string(),
                });
            }
        }

        if let Some(reverse) = &node.reverse_properties {
            for (predicate, nodes) in reverse.iter() {
                let predicate_str = id_to_string(predicate);
                for reverse_node in nodes.iter() {
                    let source_id = self.process_node(reverse_node.as_ref());
                    self.edges.push(GraphEdge {
                        from: source_id,
                        to: node_id.clone(),
                        predicate: predicate_str.clone(),
                    });
                }
            }
        }

        for (predicate, values) in node.properties.iter() {
            let predicate_str = id_to_string(predicate);
            let collected = self.collect_property_values(&node_id, &predicate_str, values);
            if !collected.is_empty() {
                for value in collected {
                    self.add_property_value(&node_id, &predicate_str, value);
                }
            }
        }

        self.processing.remove(&node_id);
        node_id
    }

    fn collect_property_values(
        &mut self,
        source_id: &str,
        predicate: &str,
        values: &[json_ld::IndexedObject<iref::IriBuf, rdf_types::BlankIdBuf>],
    ) -> Vec<JsonValue> {
        let mut collected = Vec::new();

        for value in values {
            self.handle_value(source_id, predicate, value, &mut collected);
        }

        collected
    }

    fn handle_value(
        &mut self,
        source_id: &str,
        predicate: &str,
        value: &json_ld::IndexedObject<iref::IriBuf, rdf_types::BlankIdBuf>,
        collected: &mut Vec<JsonValue>,
    ) {
        match value.as_ref() {
            json_ld::Object::Value(v) => {
                if let Some(json) = value_object_to_json(v) {
                    collected.push(json);
                }
            }
            json_ld::Object::Node(node) => {
                let target_id = self.process_node(node);
                self.edges.push(GraphEdge {
                    from: source_id.to_string(),
                    to: target_id,
                    predicate: predicate.to_string(),
                });
            }
            json_ld::Object::List(list) => {
                let mut list_values = Vec::new();
                for item in list.iter() {
                    self.handle_value(source_id, predicate, item, &mut list_values);
                }
                if !list_values.is_empty() {
                    collected.push(JsonValue::Array(list_values));
                }
            }
        }
    }

    fn add_property_value(&mut self, node_id: &str, predicate: &str, value: JsonValue) {
        let entry = self
            .nodes
            .get_mut(node_id)
            .expect("node must exist before adding properties");
        match entry.properties.get_mut(predicate) {
            Some(existing) => {
                if existing.is_array() {
                    if let Some(arr) = existing.as_array_mut() {
                        arr.push(value);
                    }
                } else {
                    let old = existing.take();
                    *existing = JsonValue::Array(vec![old, value]);
                }
            }
            None => {
                entry.properties.insert(predicate.to_string(), value);
            }
        }
    }

    pub fn into_graph(mut self) -> KnowledgeGraph {
        for node in self.nodes.values_mut() {
            node.types.sort();
            node.types.dedup();
        }
        let mut nodes: Vec<GraphNode> = self.nodes.into_values().collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        KnowledgeGraph {
            nodes,
            edges: self.edges,
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn value_object_to_json(value: &json_ld::Value<iref::IriBuf>) -> Option<JsonValue> {
    match value {
        json_ld::Value::Literal(lit, _) => match lit {
            Literal::Null => Some(JsonValue::Null),
            Literal::Boolean(b) => Some(JsonValue::Bool(*b)),
            Literal::Number(n) => serde_json::Number::from_str(n.as_ref())
                .ok()
                .map(JsonValue::Number),
            Literal::String(s) => Some(JsonValue::String(s.to_string())),
        },
        json_ld::Value::LangString(lang) => {
            let (value, language, direction) = lang.clone().into_parts();
            let mut obj = Map::new();
            obj.insert("@value".to_string(), JsonValue::String(value.to_string()));
            if let Some(lang_tag) = language {
                obj.insert(
                    "@language".to_string(),
                    JsonValue::String(lang_tag.to_string()),
                );
            }
            if let Some(dir) = direction {
                obj.insert("@direction".to_string(), JsonValue::String(dir.to_string()));
            }
            Some(JsonValue::Object(obj))
        }
        json_ld::Value::Json(json) => Some(json_syntax_value_to_json(json)),
    }
}

fn json_syntax_value_to_json(value: &SyntaxValue) -> JsonValue {
    match value {
        SyntaxValue::Null => JsonValue::Null,
        SyntaxValue::Boolean(b) => JsonValue::Bool(*b),
        SyntaxValue::Number(n) => serde_json::Number::from_str(n.as_ref())
            .ok()
            .map(JsonValue::Number)
            .unwrap_or_else(|| JsonValue::String(n.to_string())),
        SyntaxValue::String(s) => JsonValue::String(s.to_string()),
        SyntaxValue::Array(items) => {
            JsonValue::Array(items.iter().map(json_syntax_value_to_json).collect())
        }
        SyntaxValue::Object(obj) => {
            let mut map = Map::new();
            for entry in obj.iter() {
                map.insert(
                    entry.as_key().to_string(),
                    json_syntax_value_to_json(entry.as_value()),
                );
            }
            JsonValue::Object(map)
        }
    }
}

fn node_identifier(id: Option<&json_ld::Id<iref::IriBuf, rdf_types::BlankIdBuf>>) -> String {
    match id {
        Some(json_ld::Id::Valid(json_ld::ValidId::Iri(iri))) => iri.as_str().to_string(),
        Some(json_ld::Id::Valid(json_ld::ValidId::Blank(blank))) => {
            format!("_:{}", blank.as_str())
        }
        Some(json_ld::Id::Invalid(raw)) => raw.clone(),
        None => format!("_:{}", Uuid::new_v4()),
    }
}

fn id_to_string(id: &json_ld::Id<iref::IriBuf, rdf_types::BlankIdBuf>) -> String {
    match id {
        json_ld::Id::Valid(json_ld::ValidId::Iri(iri)) => iri.as_str().to_string(),
        json_ld::Id::Valid(json_ld::ValidId::Blank(blank)) => {
            format!("_:{}", blank.as_str())
        }
        json_ld::Id::Invalid(raw) => raw.clone(),
    }
}
