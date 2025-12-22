use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::graph::{CodeGraph, EdgeType, NodeType};

pub struct GraphExporter;

impl GraphExporter {
    /// Export graph to GraphML format
    pub fn export_graphml(graph: &CodeGraph, output: &Path) -> Result<()> {
        let mut file = File::create(output)?;

        writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        writeln!(
            file,
            "<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\""
        )?;
        writeln!(
            file,
            "         xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\""
        )?;
        writeln!(
            file,
            "         xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns"
        )?;
        writeln!(
            file,
            "         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">"
        )?;

        // Define attributes
        writeln!(
            file,
            "  <key id=\"name\" for=\"node\" attr.name=\"name\" attr.type=\"string\"/>"
        )?;
        writeln!(
            file,
            "  <key id=\"type\" for=\"node\" attr.name=\"type\" attr.type=\"string\"/>"
        )?;
        writeln!(
            file,
            "  <key id=\"file\" for=\"node\" attr.name=\"file_path\" attr.type=\"string\"/>"
        )?;
        writeln!(
            file,
            "  <key id=\"exported\" for=\"node\" attr.name=\"is_exported\" attr.type=\"boolean\"/>"
        )?;
        writeln!(
            file,
            "  <key id=\"used\" for=\"node\" attr.name=\"is_used\" attr.type=\"boolean\"/>"
        )?;
        writeln!(
            file,
            "  <key id=\"edge_type\" for=\"edge\" attr.name=\"edge_type\" attr.type=\"string\"/>"
        )?;

        writeln!(file, "  <graph id=\"G\" edgedefault=\"directed\">")?;

        // Write nodes
        for (id, node) in &graph.nodes {
            writeln!(file, "    <node id=\"n{}\">", id)?;
            writeln!(
                file,
                "      <data key=\"name\">{}</data>",
                escape_xml(&node.name)
            )?;
            writeln!(file, "      <data key=\"type\">{:?}</data>", node.node_type)?;
            writeln!(
                file,
                "      <data key=\"file\">{}</data>",
                escape_xml(&node.file_path.display().to_string())
            )?;
            writeln!(
                file,
                "      <data key=\"exported\">{}</data>",
                node.is_exported
            )?;
            writeln!(file, "      <data key=\"used\">{}</data>", node.is_used)?;
            writeln!(file, "    </node>")?;
        }

        // Write edges
        let mut edge_count = 0;
        for edge in &graph.edges {
            // Skip edges with invalid node IDs
            if !graph.nodes.contains_key(&edge.from) || !graph.nodes.contains_key(&edge.to) {
                continue;
            }

            writeln!(
                file,
                "    <edge id=\"e{}\" source=\"n{}\" target=\"n{}\">",
                edge_count, edge.from, edge.to
            )?;
            writeln!(
                file,
                "      <data key=\"edge_type\">{:?}</data>",
                edge.edge_type
            )?;
            writeln!(file, "    </edge>")?;
            edge_count += 1;
        }

        writeln!(file, "  </graph>")?;
        writeln!(file, "</graphml>")?;

        Ok(())
    }

    /// Export graph to DOT format (Graphviz)
    pub fn export_dot(graph: &CodeGraph, output: &Path) -> Result<()> {
        let mut file = File::create(output)?;

        writeln!(file, "digraph DependencyGraph {{")?;
        writeln!(file, "  rankdir=LR;")?;
        writeln!(file, "  node [shape=box, style=rounded];")?;
        writeln!(file)?;

        // Write nodes with styling
        for (id, node) in &graph.nodes {
            let color = match node.node_type {
                NodeType::Function => "lightblue",
                NodeType::Class => "lightgreen",
                NodeType::Method => "lightyellow",
                NodeType::Variable => "lightgray",
            };

            let style = if !node.is_used {
                "filled,dashed"
            } else if node.is_exported {
                "filled,bold"
            } else {
                "filled"
            };

            writeln!(
                file,
                "  n{} [label=\"{}\\n{:?}\", fillcolor={}, style=\"{}\"];",
                id,
                escape_dot(&node.name),
                node.node_type,
                color,
                style
            )?;
        }

        writeln!(file)?;

        // Write edges with styling
        for edge in &graph.edges {
            // Skip edges with invalid node IDs
            if !graph.nodes.contains_key(&edge.from) || !graph.nodes.contains_key(&edge.to) {
                continue;
            }

            let style = match edge.edge_type {
                EdgeType::Calls => "solid",
                EdgeType::References => "dashed",
                EdgeType::Instantiates => "dotted",
                EdgeType::Imports => "bold",
            };

            writeln!(
                file,
                "  n{} -> n{} [style={}, label=\"{:?}\"];",
                edge.from, edge.to, style, edge.edge_type
            )?;
        }

        writeln!(file, "}}")?;

        Ok(())
    }

    /// Export graph to JSON format
    pub fn export_json(graph: &CodeGraph, output: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(graph)?;
        std::fs::write(output, json)?;
        Ok(())
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
