use cosmwasm_std::{Deps, DepsMut, Order, StdResult};

use cvm_route::{exchange::ExchangeItem, venue::AssetsVenueItem};
use cvm_runtime::exchange::ExchangeId;
use cw_storage_plus::{IndexedMap, Map, MultiIndex};

use crate::{
    auth,
    batch::BatchResponse,
    error::{ContractError, Result},
    events::make_event,
};

pub(crate) fn get_by_id(deps: Deps, exchange_id: ExchangeId) -> Result<ExchangeItem> {
    EXCHANGE
        .may_load(deps.storage, exchange_id.0)?
        .ok_or(ContractError::ExchangeNotFound)
}

pub fn get_all_exchanges(deps: Deps) -> StdResult<Vec<ExchangeItem>> {
    EXCHANGE
        .range(deps.storage, None, None, Order::Ascending)
        .map(|x| x.map(|(_, x)| x))
        .collect()
}

pub fn get_all_exchange_venues(deps: Deps) -> StdResult<Vec<AssetsVenueItem>> {
    EXCHANGE_VENUE
        .range(deps.storage, None, None, Order::Ascending)
        .map(|x| x.map(|(_, x)| x))
        .collect()
}


pub(crate) fn force_exchange(
    _: auth::Admin,
    deps: DepsMut,
    msg: ExchangeItem,
) -> Result<BatchResponse> {
    EXCHANGE.save(deps.storage, msg.exchange_id.0, &msg)?;
    Ok(BatchResponse::new().add_event(
        make_event("exchange.forced").add_attribute("exchange_od", msg.exchange_id.to_string()),
    ))
}

pub(crate) const EXCHANGE: Map<u128, cvm_route::exchange::ExchangeItem> = Map::new("exchange");

pub type VenuePairId = (u128, u128);

pub type VenueMultiMap<'a> =
    IndexedMap<'a, (VenuePairId, u128), cvm_route::venue::AssetsVenueItem, VenueIndexes<'a>>;

pub struct VenueIndexes<'a> {
    pub pair_first: MultiIndex<'a, VenuePairId, cvm_route::venue::AssetsVenueItem, (VenuePairId, u128,)>,
}

impl<'a> cw_storage_plus::IndexList<cvm_route::venue::AssetsVenueItem> for VenueIndexes<'a> {
    fn get_indexes(
        &'_ self,
    ) -> Box<
        dyn Iterator<Item = &'_ dyn cw_storage_plus::Index<cvm_route::venue::AssetsVenueItem>> + '_,
    > {
        let v: Vec<&dyn cw_storage_plus::Index<cvm_route::venue::AssetsVenueItem>> =
            vec![&self.pair_first];
        Box::new(v.into_iter())
    }
}

pub const fn venues<'a>() -> VenueMultiMap<'a> {
    let indexes = VenueIndexes {
        pair_first: MultiIndex::new(
            |_pk: &[u8], d: &cvm_route::venue::AssetsVenueItem| {
                (d.from_asset_id.into(), d.to_asset_id.into())
            },
            "exchange_id_pair",
            "pair",
        ),
    };
    IndexedMap::new("venues", indexes)
}

pub const EXCHANGE_VENUE: IndexedMap<(VenuePairId, u128), cvm_route::venue::AssetsVenueItem, VenueIndexes> =
    venues();

pub(crate) fn force_assets_venue(
    _: auth::Admin,
    deps: DepsMut,
    msg: AssetsVenueItem,
) -> Result<BatchResponse> {
    match msg.venue_id {
        cvm_route::venue::VenueId::Exchange(exchange_id) => {
            EXCHANGE_VENUE.save(
                deps.storage,
                ((msg.from_asset_id.0.0, msg.to_asset_id.0.0), exchange_id.0),
                &msg,
            )?;
            Ok(BatchResponse::new().add_event(
                make_event("venue.forced")
                    .add_attribute("from_asset_id", msg.from_asset_id.to_string())
                    .add_attribute("to_asset_id", msg.to_asset_id.to_string()),
            ))
        }
        cvm_route::venue::VenueId::Transfer => panic!("no special handling for transfer"),
    }
}
