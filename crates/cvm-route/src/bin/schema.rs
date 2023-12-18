#[cfg(all(feature = "json-schema", not(target_arch = "wasm32")))]
#[allow(clippy::disallowed_methods)]
fn main() {
    use ::std::{fs, fs::write};
    use cvm_route::asset::*;
    use cvm_route::transport::*;
    use cvm_route::exchange::*;
    #[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
    pub enum CvmRouteSchema {
        NetworkToNetwork(NetworkToNetwork),
        AssetToNetwork(AssetItem),
        ExchangeItem(ExchangeItem),
    }

    let root = schemars::gen::SchemaGenerator::default().into_root_schema_for::<CvmRouteSchema>();

    let mut out_dir = std::env::current_dir().unwrap();
    out_dir.push("schema");

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).expect("Failed to create directory");
    }

    let path = out_dir.join(concat!("cvm-route", ".json"));
    
    write(&path, serde_json::to_string(&root).unwrap()).unwrap();
}

#[cfg(not(all(feature = "json-schema", not(target_arch = "wasm32"))))]

fn main() {}
