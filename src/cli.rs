use clap::Parser;
use std::path::PathBuf;
/// Check property on transitions in mealy machine dot file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// dot file to be verified
    #[arg(action=clap::ArgAction::Append)]
    pub(crate) graphs: Vec<String>,

    /// rules to check against the mealy machines
    #[arg(short, long)]
    pub(crate) rules: String,
    ///Output folder, if not provided a random name is chosen
    #[arg(short, long)]
    pub(crate) output_folder: Option<PathBuf>,
}
