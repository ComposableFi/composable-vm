#[cfg(all(feature = "json-schema", not(target_arch = "wasm32")))]
#[allow(clippy::disallowed_methods)]
fn main() {
    use cosmwasm_schema::write_api;
    use cvm_runtime::outpost;

    write_api! {
        instantiate: outpost::InstantiateMsg,
        query: outpost::QueryMsg,
        execute: outpost::ExecuteMsg,
    }
}

#[cfg(not(all(feature = "json-schema", not(target_arch = "wasm32"))))]

fn main() {}
