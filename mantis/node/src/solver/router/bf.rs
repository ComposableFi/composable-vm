use cvm_runtime::shared::CvmProgram;
use petgraph::algo::{dijkstra, min_spanning_tree};
use petgraph::data::FromElements;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{NodeIndex, UnGraph};

pub fn route(
    input: crate::mantis::solve::IntentBankInput,
    cvm_glt: &cvm_runtime::outpost::GetConfigResponse,
    salt: &[u8],
) -> CvmProgram {
    todo!()
}
