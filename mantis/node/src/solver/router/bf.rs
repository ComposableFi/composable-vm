use std::collections::BTreeMap;

use blackbox_rs::types::SingleInputAssetCvmRoute;
use cvm_route::venue::VenueId;
use cvm_runtime::proto::pb::program::{Exchange, Transfer};
use cvm_runtime::shared::{CvmInstruction, CvmProgram};
use cvm_runtime::{exchange, AssetId, ExchangeId};
use petgraph::algo::{bellman_ford, min_spanning_tree};
use petgraph::data::FromElements;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{NodeIndex, UnGraph};

// need some how unify with python
pub enum Venue {
    Transfer(AssetId, AssetId),
    Exchange(ExchangeId, AssetId, AssetId),
}

pub fn get_all_asset_maps(cvm_glt: &cvm_runtime::outpost::GetConfigResponse) -> Vec<Venue> {
    let transfers = cvm_glt
        .network_assets
        .iter()
        // CVM GLT has 2 entires for each direction for bidirectional transfers
        .map(|x| Venue::Transfer(x.from_asset_id, x.to_asset_id));
    let exchanges = cvm_glt.asset_venue_items.iter().map(|x| match x.venue_id {
        VenueId::Transfer => panic!(),
        VenueId::Exchange(exchange_id) => Venue::Exchange(exchange_id, x.from_asset_id, x.to_asset_id),
    });

    transfers.chain(exchanges).collect()
}

pub fn route(
    cvm_glt: &cvm_runtime::outpost::GetConfigResponse,
    input: crate::mantis::solve::IntentBankInput,
    salt: &[u8],
) -> CvmInstruction {
    let mut graph = petgraph::graph::DiGraph::new();
    let mut assets_global_to_local = std::collections::BTreeMap::new();
    for asset_id in cvm_glt.get_all_asset_ids() {
        let node = graph.add_node(1);
        assets_global_to_local.insert(asset_id, node);
    }

    let mut venue_local_to_global = BTreeMap::new();
    for venue in get_all_asset_maps(cvm_glt) {
        match venue {
            Venue::Transfer(from, to) => {
                let from_node = assets_global_to_local.get(&from).unwrap();
                let to_node = assets_global_to_local.get(&to).unwrap();
                let local_venue = graph.add_edge(*from_node, *to_node, 1.0);
                venue_local_to_global.insert(local_venue, venue.clone());
            }
            Venue::Exchange(exchange_id, from, to) => {
                let from_node = assets_global_to_local.get(&from).unwrap();
                let to_node = assets_global_to_local.get(&to).unwrap();
                let local_venue = graph.add_edge(*from_node, *to_node, 1.0);
                venue_local_to_global.insert(local_venue, venue.clone());
            }
        }
    }

    let in_node_index = assets_global_to_local.get(&input.in_asset_id).unwrap();
    let routes =
        bellman_ford::bellman_ford(&graph, *in_node_index)
            .expect("bf");
    let out_node_index = assets_global_to_local.get(&input.out_asset_id).expect("node");
    
    
    let mut out_node_index = out_node_index.index();
    let mut in_node_index = routes.predecessors[out_node_index].expect("routable");
    while true {

    }

    panic!()
}
