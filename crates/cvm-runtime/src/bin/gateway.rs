#[cfg(all(feature = "json-schema", not(target_arch = "wasm32")))]
#[allow(clippy::disallowed_methods)]
fn main() {
    use cosmwasm_schema::write_api;
    use cvm_runtime::gateway;

    write_api! {
        instantiate: gateway::InstantiateMsg,
        query: gateway::QueryMsg,
        execute: gateway::ExecuteMsg,
    }
}

#[cfg(not(all(feature = "json-schema", not(target_arch = "wasm32"))))]

fn main() {}
