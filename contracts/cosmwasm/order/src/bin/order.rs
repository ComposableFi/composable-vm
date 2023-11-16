#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::disallowed_methods)]
fn main() {
    use cosmwasm_schema::write_api;
    use cw_mantis_order::sv::*;
    use cw_mantis_order::*;
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecMsg,
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
