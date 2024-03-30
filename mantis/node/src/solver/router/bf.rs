use cvm_runtime::shared::CvmProgram;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::algo::{dijkstra, min_spanning_tree};
use petgraph::data::FromElements;
use petgraph::dot::{Dot, Config};

pub fn route(input: crate::mantis::solve::BankInput, cvm_glt: &cvm_runtime::outpost::GetConfigResponse, salt: &[u8]) -> CvmProgram {
    todo!()
}

