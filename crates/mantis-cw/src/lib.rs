
mod ordered_tuple;
mod ordered_coin_pair;

pub use ordered_tuple::OrderedTuple2;
pub use ordered_coin_pair::OrderCoinPair;

pub type Denom = String;
pub type DenomPair = OrderedTuple2<String>;