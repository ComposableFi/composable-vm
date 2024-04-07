use blackbox_rs::types::SingleInputAssetCvmRoute;
use cvm_runtime::proto::pb::program::{Exchange, Transfer};
use cvm_runtime::shared::CvmProgram;
use cvm_runtime::{AssetId, ExchangeId};
use petgraph::algo::{bellman_ford, dijkstra, min_spanning_tree};
use petgraph::data::{Build, FromElements};
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
        Transfer => panic!(),
        Exchange => Venue::Exchange(x.exchange_id, x.from_asset_id, x.to_asset_id),
    });

    transfers.chain(exchanges).collect()
}

pub fn route(
    cvm_glt: &cvm_runtime::outpost::GetConfigResponse,
    input: crate::mantis::solve::IntentBankInput,
) -> SingleInputAssetCvmRoute {
    let mut graph = petgraph::graph::DiGraph::new();
    let mut assets_global_to_local = std::collections::BTreeMap::new();
    for asset_id in cvm_glt.network_assets.iter() {
        let node = graph.add_node(1);
        assets_global_to_local.insert(asset_id, node);
    }

    for venue in get_all_asset_maps(cvm_glt) {
        match venue {
            Venue::Transfer(from, to) => {
                let from_node = assets_global_to_local.get(&from).unwrap();
                let to_node = assets_global_to_local.get(&to).unwrap();
                graph.add_edge(*from_node, *to_node, 1);
            }
            Venue::Exchange(exchange_id, from, to) => {
                let from_node = assets_global_to_local.get(&from).unwrap();
                let to_node = assets_global_to_local.get(&to).unwrap();
                graph.add_edge(*from_node, *to_node, 1);
            }
        }
    }

    let route =
        bellman_ford::bellman_ford(graph, assets_global_to_local.get(&input.asset_id).unwrap())
            .expect("bf");
}
