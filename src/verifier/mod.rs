use std::{fs, path::PathBuf};

use crate::{cli::Args, graph::Graph};

use self::rules::Rule;

pub(crate) mod event;
pub(crate) mod premise;
pub(crate) mod rules;

pub(crate) struct Verifier {
    pub(crate) rules: Vec<Box<dyn Rule>>,
    pub(crate) graphs: Vec<Graph>,
    pub(crate) output_folder: PathBuf,
}

impl Verifier {
    pub(crate) fn from_args(args: Args) -> Self {
        if args.graphs.is_empty(){
            panic!("dot file is required.")
        }
        let rules = rules::parse_file(&args.rules);
        let mut graphs: Vec<Graph> = Vec::with_capacity(args.graphs.len());
        for path_to_graph_file in args.graphs {
            graphs.push(Graph::new_file(&path_to_graph_file));
        }
        let output_folder = match args.output_folder {
            None => {
                let result_folder_number = rand::random::<u8>();
                println!("Output folder is  result_{}", result_folder_number);
                PathBuf::from(format!("result_{}", result_folder_number))
            }
            Some(p) => p,
        };
        Verifier {
            rules,
            graphs,
            output_folder,
        }
    }

    pub(crate) fn apply(&mut self) {
        for graph in self.graphs.iter() {
            self.output_folder.push(graph.get_name());
            // println!("graph: {}",graph.get_name());
            if let Err(e) = fs::create_dir_all(&self.output_folder) {
                panic!("failed to create output directory due to :{}", e.to_string());
            }
            for r in self.rules.iter_mut() {
                // println!("rule:{},",r.get_name());
                r.apply(graph, &mut self.output_folder);
            }
            self.output_folder.pop();
        }
    }
}
