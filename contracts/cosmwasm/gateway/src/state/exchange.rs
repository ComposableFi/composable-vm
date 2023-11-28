use cvm_runtime::service::dex::ExchangeItem;
use cw_storage_plus::Map;

pub(crate) const EXCHANGE: Map<u128, ExchangeItem> = Map::new("exchange");
