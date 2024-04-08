use cosmwasm_std::DepsMut;
use cvm_route::venue::AssetsVenueItem;

use crate::{auth::Admin, prelude::*};

pub(crate) fn force_assets_venue(_: Admin, deps: DepsMut, msg: AssetsVenueItem) -> Result<crate::batch::BatchResponse, crate::error::ContractError> {
    todo!()
}