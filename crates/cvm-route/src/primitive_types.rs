//! paritytech/primitive-types copy until they support std wasm compiled or CW support no_std
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
pub struct H160(
    #[serde(
        serialize_with = "hex::serialize",
        deserialize_with = "hex::deserialize"
    )]
    #[cfg_attr(feature = "json-schema", schemars(schema_with = "String::json_schema"))]
    [u8; 20],
);

impl H160 {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
