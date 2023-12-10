const MIN_RESERVE: f64 = 05000.0;
const MAX_RESERVE: f64 = 15000.0;

use std::collections::HashMap;

use cosmrs::tendermint::chain;
use good_lp::*;
use itertools::*;
use ndarray::*;
use prelude::*;
use tuples::TupleCloned;

pub fn populate_chain_dict(chains: &mut HashMap<String, Vec<String>>, center_node: String) {
    let others: HashMap<_, _> = chains
        .clone()
        .into_iter()
        .filter(|(chain, _)| *chain != center_node)
        .collect();

    for (chain, tokens) in others.iter() {
        let mut center = chains.entry(center_node.clone());
        let center = center.or_insert(<_>::default());
        for token in tokens {
            center.push(format!("{}/{}", chain, token))
        }
    }

    let center_tokens = chains.get(&center_node).unwrap().clone();

    for (chain, tokens) in chains.iter_mut() {
        if chain != &center_node {
            for token in center_tokens.iter() {
                if !token.contains(chain) {
                    tokens.push(format!("{}/{}", center_node, token));
                }
            }
        }
    }
}

pub fn solve(
    all_tokens: Vec<String>,
    all_cffms: Vec<(String, String)>,
    reserves: ndarray::Array1<f64>,
    cfmm_tx_cost: Vec<f64>,
    fees: Vec<f64>,
    ibc_pools: u16,
    origin_token: String,
    number_of_init_tokens: u128,
    obj_token: String,
    force_eta: Vec<f64>,
) {
    let count_tokens = all_tokens.len();
    let count_cffms = all_cffms.len();
    let current_assets = ndarray::Array1::<f64>::from_elem(count_tokens, <_>::default());
}
