use crate::{prelude::*, state, state::State};
use cvm_runtime::{shared::*, InterpreterOrigin, Register};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {
    /// Owners to be added to the list of owners which acts more like a recovery in case all of the
    /// owners are erased accidentally
    pub owners: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema, cosmwasm_schema::QueryResponses)
)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get a specific register
    #[cfg_attr(feature = "json-schema", returns(QueryStateResponse))]
    Register(Register),
    /// dumps the whole state of interpreter
    #[cfg_attr(feature = "json-schema", returns(QueryStateResponse))]
    State(),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct QueryStateResponse {
    pub state: state::State,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct QueryExchangeResponse {
    pub state: State,
}
