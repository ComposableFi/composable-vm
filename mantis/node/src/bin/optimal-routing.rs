use itertools::*;
use mantis_node::solver::router::*;
use std::collections::HashMap;
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
    for (other_chain, other_tokens) in chains.clone() {
        all_tokens.extend(other_tokens.clone());
        let cfmms = other_tokens.clone().into_iter().combinations(2);
        all_cfmms.extend(cfmms);
    }
}
