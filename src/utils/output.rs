use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::graph::edgemap::EdgeMap;
use crate::graph::multi_edge::MultiEdge;
use crate::graph::nodeid::NodeId;
use crate::graph::Graph;

pub(crate) fn write_files(
    graph: &Graph,
    nodes: HashSet<NodeId>,
    edges: HashSet<(&MultiEdge, Vec<usize>)>,
    output_folder: &mut PathBuf,
) -> std::io::Result<()> {
    if nodes.is_empty() && edges.is_empty() {
        return Ok(());
    }
    output_folder.push("ce.dot");
    let mut output = String::from("digraph \"Automata\" { \n");
    let mut file = match fs::OpenOptions::new().write(true).truncate(true).open(&output_folder) {
        Ok(f) => f,
        Err(_) => File::create(&output_folder)?,
    };
    for node_id in nodes.iter() {
        //the id comes from the same graph hence it must be there
        let node_line = graph.get_node(node_id).unwrap().to_string();
        output.push_str(&node_line);
    }
    for (edge, indexes) in edges.into_iter() {
        let edge_lines = edge.to_string_labels(indexes.into_iter());
        for s in edge_lines {
            output.push_str(&s);
        }
    }
    output.push_str("}\n");
    file.write_all(output.as_bytes())?;
    output_folder.pop();
    Ok(())
}

pub(crate) fn write_files_edge_map(
    graph: &Graph,
    nodes: HashSet<NodeId>,
    edges: EdgeMap,
    output_folder: &mut PathBuf,
) -> std::io::Result<()> {
    if nodes.is_empty() && edges.is_empty() {
        return Ok(());
    }
    output_folder.push("ce.dot");
    let mut output = String::from("digraph \"Automata\" { \n");
    let mut file = match fs::OpenOptions::new().write(true).truncate(true).open(&output_folder) {
        Ok(f) => f,
        Err(_) => File::create(&output_folder)?,
    };
    for node_id in nodes.iter() {
        //the id comes from the same graph hence it must be there
        let node_line = graph.get_node(node_id).unwrap().to_string();
        output.push_str(&node_line);
    }
    output.push_str(&edges.to_string());
    output.push_str("}\n");
    file.write_all(output.as_bytes())?;
    output_folder.pop();
    Ok(())
}