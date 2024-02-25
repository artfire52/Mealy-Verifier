use indexmap::IndexSet;

use crate::{
    graph::{edgemap::EdgeMap, multi_edge::MultiEdge, nodeid::NodeId, Graph},
    utils::{self, reader::Reader},
    verifier::{event::Events, rules::Rule},
};
#[cfg(test)]
use std::any::Any;
use std::{
    collections::HashSet, fs, rc::Rc
};

#[derive(Debug)]
pub(crate) struct UnexpectedCycle {
    pub(crate) cycle: Events,
    pub(crate) name: String,
}

impl UnexpectedCycle {
    fn inner_apply<'a>(&'a self, graph: &Graph)->(HashSet<NodeId>, EdgeMap){
        let mut output_nodes:HashSet<NodeId>=HashSet::new();
        let mut output_edges=EdgeMap::new();
        for node in graph.iter_node_id(){
            self.apply_on_node(node, graph, &mut output_nodes, &mut output_edges);
        }
        (output_nodes,output_edges)
    }

    fn apply_on_node<'a>(&'a self, source_node:&NodeId, graph: &Graph, output_nodes:&mut HashSet<NodeId>, output_edges:&mut EdgeMap) {
        let mut execution_stack: Vec<(NodeId, Vec<(NodeId, &MultiEdge)>, usize,Option<(&MultiEdge,Vec<usize>)>)> = Vec::new();
        let neighbors=graph.neighbors_edges_iterator(source_node);
        execution_stack.push((source_node.clone(),neighbors,0,None));
        let mut path:IndexSet<(NodeId,Option<(&MultiEdge,Vec<usize>)>)>=IndexSet::new();
        let mut seen: HashSet<(NodeId,usize)>=HashSet::new();
        let cycle_len=self.cycle.len();
        while let Some((node_id,neighbors,index,from)) =execution_stack.last_mut()  {
            seen.insert((node_id.clone(),index.clone()));
            path.insert((node_id.clone(),from.clone()));
            if node_id==source_node && *index==cycle_len{
                let extra_node=(node_id.clone(),from.clone().unwrap());
                Self::add_nodes_and_edges(&path,  output_nodes, output_edges,extra_node);
                //the index will not increase anymore so we can go to the previous node.
                execution_stack.pop();
                path.pop();
                continue
            }
            if let Some((dest_node_id, edge))=neighbors.pop(){
                let new_index=*index+1;
                if new_index>cycle_len{
                    continue
                }
                let mut indexes=Vec::new();
                let mut indexes_seen=Vec::new();
                for (label_index,label) in edge.get_label_iterator().enumerate(){
                    if !seen.contains(&(dest_node_id.clone(),(*index+1))) {
                        if self.check_cycle_index(*index,label){
                            indexes.push(label_index);
                        }
                    }else{
                        if self.check_cycle_index(*index,label){
                            indexes_seen.push(label_index);
                        }
                      
                    }

                }
                if !indexes.is_empty(){
                    execution_stack.push((dest_node_id.clone(),graph.neighbors_edges_iterator(&dest_node_id),new_index, Some((edge,indexes))));
                }
                if !indexes_seen.is_empty(){
                    let extra_node=(dest_node_id.clone(),(edge,indexes_seen));
                    Self::add_nodes_and_edges(&path,  output_nodes, output_edges,extra_node);
                }
            }else{
                execution_stack.pop();
                path.pop();
            }
        }


    }


    fn add_nodes_and_edges(
        path_node: &IndexSet<(NodeId,Option<(&MultiEdge,Vec<usize>)>)>,
        output_node: &mut HashSet<NodeId>,
        output_edges: &mut EdgeMap, 
        extra_node:(NodeId, (&MultiEdge, Vec<usize>))
    ) {
        for chunk in path_node.iter().collect::<Vec<_>>().windows(2) {
                //pattern matching chunk
                let (from_id,_) = &chunk[0];
                let (to_id,edges) = &chunk[1];
                let (edge,indexes) = edges.as_ref().unwrap();

                //add the nodes to the output
                output_node.insert(from_id.clone());
                output_node.insert(to_id.clone());

                //add the corresponding edge
                output_edges.add_edge_with_indexes(edge,indexes);
        }
        output_node.insert(extra_node.0.clone());
        output_edges.add_edge_with_indexes(extra_node.1.0,&extra_node.1.1);

        
    }

    fn check_cycle_index(&self,index:usize,label:&Rc<str>,)->bool{
        if self.cycle.check(index, label){
            return true;
        }
        false
    }
}

impl Rule for UnexpectedCycle {
    fn from_reader(reader: &mut dyn Reader, name: String) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut cycle: Option<Events> = None;
        while let Some(line) = reader.read_line() {
            if line.starts_with(":UC") {
                break;
            } else if line.contains('/') {
                cycle = Some(Events::from_str(line));
            } else {
                panic!("failed to parse unexpected loop rule");
            }
        }
        let cycle = match cycle {
            Some(c) => c,
            None => panic!("failed to parse expected cycle rule"),
        };
        Ok(UnexpectedCycle { cycle, name })
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn apply(
        &mut self,
        graph: &Graph,
        output_folder: &mut std::path::PathBuf,
    ) {
        let (nodes, edges) = self.inner_apply(graph);
        output_folder.push(self.get_name());
        if let Err(e) = fs::create_dir_all(&output_folder) {
            panic!("failed to create output directory due to :{}", e.to_string());
        }
        let ret = utils::output::write_files_edge_map(graph, nodes, edges, output_folder);
        if let Err(e) = ret {
            panic!(
                "unable to output the result of rule {} due to {}",
                self.get_name(),
                e.to_string()
            );
        }
        output_folder.pop();
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    //https://dot-to-ascii.ggerganov.com/
    use crate::{
        graph::{edgemap::EdgeMap, nodeid::NodeId, Graph},
        verifier::rules::unexpected_cycle::UnexpectedCycle,
    };

    #[test]
    fn test_unexpected_cycle() {
        let input = r#"digraph "Automata" { 
            "0" [shape=ellipse, style=filed, fillcolor=white, URL="0"];
            "1" [shape=ellipse, style=filed, fillcolor=white, URL="1"];
            "2" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "3" [shape=ellipse, style=filed, fillcolor=white, URL="456"];
            "0" -> "1" [fontsize=5, label="0/1", color=black];
            "0" -> "1" [fontsize=5, label="0/osef", color=black];
            "1" -> "0" [fontsize=5, label="1/0", color=black];
            "2" -> "0" [fontsize=5, label="2/0", color=black];
            "1" -> "2" [fontsize=5, label="1/2", color=black];
            "1" -> "2" [fontsize=5, label="1/osef", color=black];
            "1" -> "3" [fontsize=5, label="1/3", color=black];
            "3" -> "0" [fontsize=5, label="3/0", color=black];
        }"#;
        //         ┌─────────────────────────┐  3/0
        //   ┌───▶ │            0            │ ◀────────┐
        //   │     └─────────────────────────┘          │
        //   │       │       │          ▲               │
        //   │       │ 0/1   │ 0/osef   │ 1/0           │
        //   │       ▼       ▼          │               │
        //   │     ┌─────────────────────────┐  1/3   ┌───┐
        //   │ 2/0 │            1            │ ─────▶ │ 3 │
        //   │     └─────────────────────────┘        └───┘
        //   │       │       │
        //   │       │ 1/2   │ 1/osef
        //   │       ▼       │
        //   │     ┌──────┐  │
        //   └──── │  2   │ ◀┘
        //         └──────┘
        let graph: Graph = Graph::new(input, false);

        let rule_text = r#"UC:test
        0/1;1/2;2/0
        :UC"#;
        let rule = crate::verifier::rules::parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<UnexpectedCycle>()
            .expect("expect UnexpectedCycle rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        let node_ids = vec!["0", "1", "2"];
        let node_ids: Vec<NodeId> = node_ids.iter().map(|e| NodeId::new(e)).collect();
        assert_eq!(nodes.len(), node_ids.len());
        let edges_expected = r#"
        "0"->"1" [fontsize=5,label="0/1"];
        "1"->"2" [fontsize=5,label="1/2"];
        "2"->"0" [fontsize=5,label="2/0"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert!(expected_edge_map.eq(&edges));

        let rule_text = r#"UC:test
        0/1;1/2;2/d
        :UC"#;
        let rule = crate::verifier::rules::parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<UnexpectedCycle>()
            .expect("expect UnexpectedCycle rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        assert!(nodes.is_empty());
        assert!(edges.is_empty());

        let rule_text = r#"UC:test
        0/1;1/2;2/0;0/osef;1/osef;2/0;0/1;1/3;3/0
        :UC"#;
        let rule = crate::verifier::rules::parse_rule_from_str(rule_text);
        let rule = rule[0]
            .as_any()
            .downcast_ref::<UnexpectedCycle>()
            .expect("expect UnexpectedCycle rule");
        let (nodes, edges) = rule.inner_apply(&graph);
        let node_ids = vec!["0", "1", "2","3"];
        let node_ids: Vec<NodeId> = node_ids.iter().map(|e| NodeId::new(e)).collect();
        assert_eq!(nodes.len(), node_ids.len());
        let edges_expected = r#"
        "0" -> "1" [fontsize=5, label="0/1"];
        "0" -> "1" [fontsize=5, label="0/osef"];
        "2" -> "0" [fontsize=5, label="2/0"];
        "1" -> "2" [fontsize=5, label="1/2"];
        "1" -> "2" [fontsize=5, label="1/osef"];
        "1" -> "3" [fontsize=5, label="1/3"];
        "3" -> "0" [fontsize=5, label="3/0"];"#;
        let edges_expected = edges_expected.split(";\n").into_iter().map(|e| e.trim());
        let mut expected_edge_map = EdgeMap::new();
        for edge_str in edges_expected {
            expected_edge_map.add_edge_from_str(edge_str);
        }
        assert!(expected_edge_map.eq(&edges));
    }
}
