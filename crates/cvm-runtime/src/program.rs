use crate::prelude::*;

#[cfg_attr(
    all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Program<Instructions> {
    /// In JSON, hex encoded identifiers to identify the program off chain (for example in
    /// indexer).
    #[serde(
        serialize_with = "hex::serialize",
        deserialize_with = "hex::deserialize"
    )]
    #[cfg_attr(
        all(feature = "json-schema", not(target_arch = "wasm32")),
        schemars(schema_with = "String::json_schema")
    )]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tag: Vec<u8>,
    /// list of instructions to be executed
    pub instructions: Instructions,
}
