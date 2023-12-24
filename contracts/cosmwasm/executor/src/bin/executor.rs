#[cfg(all(feature = "json-schema", not(target_arch = "wasm32")))]
#[allow(clippy::disallowed_methods)]
fn main() {
    use cosmwasm_schema::write_api;
    use cvm_runtime::executor::*;
    use cw_cvm_executor::msg::*;

    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
    let events = schemars::gen::SchemaGenerator::default()
        .into_root_schema_for::<cw_cvm_executor::events::CvmExecutor>();

    // same as in above macro
    let mut out_dir = std::env::current_dir().unwrap();
    out_dir.push("schema");

    use ::std::fs::write;

    let path = out_dir.join(concat!("events", ".json"));

    write(&path, serde_json_wasm::to_string(&events).unwrap()).unwrap();
}

#[cfg(not(all(feature = "json-schema", not(target_arch = "wasm32"))))]
fn main() {}
