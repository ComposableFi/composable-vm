use cw_storage_plus::Map;

pub(crate) const EXCHANGE: Map<u128, cvm_route::exchange::ExchangeItem> = Map::new("exchange");
