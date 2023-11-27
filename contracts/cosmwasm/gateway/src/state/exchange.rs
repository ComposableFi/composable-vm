use cw_storage_plus::Map;
use cvm_runtime::service::dex::ExchangeItem;

pub(crate) const EXCHANGE: Map<u128, ExchangeItem> = Map::new("exchange");
