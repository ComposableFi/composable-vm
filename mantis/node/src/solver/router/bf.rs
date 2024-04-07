use blackbox_rs::types::SingleInputAssetCvmRoute;
use cvm_runtime::shared::CvmProgram;
use cvm_runtime::{AssetId, ExchangeId};
use petgraph::algo::{dijkstra, min_spanning_tree};
use petgraph::data::{Build, FromElements};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{NodeIndex, UnGraph};


// need some how unify with python
pub enum Venue{
    Transfer(AssetId, AssetId),
    Exchange(ExchangeId, AssetId, AssetId)
}


pub fn get_all_asset_maps(cvm_glt: &cvm_runtime::outpost::GetConfigResponse) -> Vec<Venue> {
    let transfers = self
        .network_assets
        .iter()
        // CVM GLT has 2 entires for each direction for bidirectional transfers
        .map(|x| (x.asset_id, x.to_asset_id));
    let exchanges = self
        .asset_venue_items
        .iter()
        .map(|x| (x.from_asset_id, x.to_asset_id));

    transfers.chain(exchanges).collect()
}

pub fn route(
    cvm_glt: &cvm_runtime::outpost::GetConfigResponse,    
    input: crate::mantis::solve::IntentBankInput,
) -> SingleInputAssetCvmRoute {
    let mut graph = petgraph::graph::DiGraph::new();
    for (from_asset_id, to_asset_id) in cvm_glt.get_all_asset_maps() {
        graph.add_edge(a, b, weight)
    }
    
}
