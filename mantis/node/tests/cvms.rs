

use std::ops::Add;

use cw_orch::prelude::*;
use cw_orch::interface;

use cw_multi_test::App;
use cosmwasm_std::Addr;

#[test]
fn cvm_devnet_case() {
    let centauri = App::default();
    let _osmosis = App::default();
    let cw_mantis_order_wasm = ContractWrapper::new(
        cw_mantis_order::entry_points::execute,
        cw_mantis_order::entry_points::instantiate,
        cw_mantis_order::entry_points::query,
    );

    let sender = Addr::unchecked("juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y");
}


