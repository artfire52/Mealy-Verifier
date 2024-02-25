mod cli;
// mod error;
mod graph;
mod utils;
mod verifier;

use crate::cli::Args;
use clap::Parser;
use verifier::Verifier;

///The Mealy verifier is a tool dedicated to analysis of Mealy machines.
///The main target of the Mealy verifier is the output of model learning of network protocol implementation.
fn main() {
    let args = Args::parse();
    let mut verifier = Verifier::from_args(args);
    verifier.apply();
}
