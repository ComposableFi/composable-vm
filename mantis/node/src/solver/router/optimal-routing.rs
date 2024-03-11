use itertools::*;
use mantis_node::solver::router::*;
use std::collections::HashMap;
use tuples::TupleCloned;
fn main() {
    let center_node = "CENTAURI";

    let mut chains: HashMap<String, Vec<String>> = HashMap::new();
    chains.insert(
        "ETHEREUM".to_owned(),
        vec!["WETH".to_owned(), "USDC".to_owned(), "SHIBA".to_owned()],
    );
    chains.insert(center_node.to_owned(), vec![]);
    chains.insert(
        "OSMOSIS".to_owned(),
        vec!["ATOM".to_owned(), "SCRT".to_owned()],
    );

    populate_chain_dict(&mut chains, center_node.to_owned());
    println!("{:?}", chains);

    let mut all_tokens = vec![];
    let mut all_cfmms = vec![];
    let mut reserves = vec![];
    let mut cfmm_tx_cost = vec![];
    for (other_chain, other_tokens) in chains.clone() {
        all_tokens.extend(other_tokens.clone());
        let cfmms = other_tokens
            .clone()
            .into_iter()
            .combinations(2)
            .map(|x| (x[0].clone(), x[1].clone()));
        all_cfmms.extend(cfmms);
    }
    use rand::prelude::*;
    let tx_costs_random = rand_distr::Uniform::new(0, 20);
    let reserves_radom = rand_distr::Uniform::new(9500, 10051);
    for cfmm in all_cfmms.iter() {
        let a = reserves_radom.sample(&mut rand::thread_rng());
        let b = reserves_radom.sample(&mut rand::thread_rng());
        reserves.push((a, b));
        let value = tx_costs_random.sample(&mut rand::thread_rng());
        cfmm_tx_cost.push(value);
    }

    let mut ibc_pools = 0u32;
    let tx_costs_random = rand_distr::Uniform::new(0, 20);
    let reserves_random = rand_distr::Uniform::new(10000, 11000);
    for token_on_center in chains.get(center_node).unwrap() {
        for (other_chain, other_tokens) in chains.iter() {
            if other_chain != center_node {
                for other_token in other_tokens {
                    if token_on_center.contains(other_token)
                        || other_token.contains(token_on_center)
                    {
                        all_cfmms.push((token_on_center.to_owned(), other_token.to_owned()));
                        let a = reserves_random.sample(&mut rand::thread_rng());
                        let b = reserves_random.sample(&mut rand::thread_rng());
                        reserves.push((a, b));
                        cfmm_tx_cost.push(tx_costs_random.sample(&mut rand::thread_rng()));
                        ibc_pools += 1;
                    }
                }
            }
        }
    }

    let mut fees = vec![];
    let fees_random = rand_distr::Uniform::new(0.97, 0.999);
    for cfmm in 0..all_cfmms.len() {
        let value = fees_random.sample(&mut rand::thread_rng());
        fees.push(value);
    }

    println!("{:?}", reserves);

    for item in all_tokens.iter().enumerate() {
        println!("{:?}", item);
    }

    for item in all_cfmms.iter().enumerate() {
        println!("{:?}", item);
    }
}
