use std::collections::HashMap;

use mantis_node::solver::router::*;
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
}
