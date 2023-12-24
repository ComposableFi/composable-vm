use cosmwasm_std::{Deps, StdResult};
use cvm_runtime::ExecutorOrigin;
use cw_storage_plus::Item;

use crate::prelude::*;

pub type ExecutorId = cvm_runtime::shared::Displayed<u128>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "json-schema", // all(feature = "json-schema", not(target_arch = "wasm32")),
    derive(schemars::JsonSchema)
)]
pub(crate) struct ExecutorItem {
    /// contract address
    pub address: Addr,
    pub executor_id: ExecutorId,
}

pub(crate) fn get_by_origin(deps: Deps, origin: ExecutorOrigin) -> StdResult<ExecutorItem> {
    let id = EXECUTOR_ORIGIN_TO_ID.load(deps.storage, origin)?;
    EXECUTORS.load(deps.storage, id)
}

pub(crate) const EXECUTORS_COUNT: Item<u128> = Item::new("executor_count");

pub(crate) const EXECUTOR_ORIGIN_TO_ID: Map<ExecutorOrigin, u128> =
    Map::new("executors_origin_to_id");

pub(crate) const EXECUTORS: Map<u128, ExecutorItem> = Map::new("executors");
