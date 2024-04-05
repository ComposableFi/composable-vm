mod ordered_coin_pair;
mod ordered_tuple;

pub use ordered_coin_pair::OrderCoinPair;
pub use ordered_tuple::OrderedTuple2;

pub type Denom = String;
pub type DenomPair = OrderedTuple2<String>;
