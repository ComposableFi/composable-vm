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
