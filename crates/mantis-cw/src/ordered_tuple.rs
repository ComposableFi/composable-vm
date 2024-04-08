use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
pub use tuples::*;

/// Ordered tuple can be considered as edge of undirected graph
#[cw_serde]
#[derive(Eq)]
pub struct OrderedTuple2<T> {
    // rust does not allows 'r#1'
    pub a: T,
    pub b: T,
}

impl<'a, T: Clone + Prefixer<'a> + KeyDeserialize + PrimaryKey<'a>> PrimaryKey<'a>
    for OrderedTuple2<T>
{
    type Prefix = T;

    type SubPrefix = T;

    type Suffix = ();

    type SuperSuffix = (T, T);
    fn key(&self) -> Vec<Key> {
        let mut keys = self.a.key();
        keys.extend(self.b.key());
        keys
    }
}

impl<'a, T: Clone + Prefixer<'a> + KeyDeserialize + PrimaryKey<'a>> Prefixer<'a>
    for OrderedTuple2<T>
{
    fn prefix(&self) -> Vec<Key> {
        let mut res = self.a.prefix();
        res.extend(self.b.prefix());
        res
    }
}

impl<
        'a,
        T: Clone + Prefixer<'a> + KeyDeserialize<Output = T> + PrimaryKey<'a> + PartialEq + PartialOrd,
    > KeyDeserialize for OrderedTuple2<T>
{
    type Output = OrderedTuple2<T>;

    #[inline(always)]
    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        let mut tu = value.split_off(2);
        let t_len = parse_length(&value)?;
        let u = tu.split_off(t_len);

        Ok(Self::new(T::from_vec(tu)?, T::from_vec(u)?))
    }
}

fn parse_length(value: &[u8]) -> StdResult<usize> {
    Ok(u16::from_be_bytes(
        value
            .try_into()
            .map_err(|_| StdError::generic_err("Could not read 2 byte length"))?,
    )
    .into())
}

impl<T> Tuple for OrderedTuple2<T> {
    fn arity(&self) -> usize {
        2
    }
}

impl<T> Tuple2 for OrderedTuple2<T> {
    type Item0 = T;
    type Item1 = T;
}

impl<T: PartialOrd + Clone + PartialOrd + PartialEq> From<OrderedTuple2<T>> for (T, T) {
    fn from(val: OrderedTuple2<T>) -> Self {
        (val.a, val.b)
    }
}

impl<T: PartialOrd + Clone + PartialOrd + PartialEq> OrderedTuple2<T> {
    pub fn new(a: T, b: T) -> Self {
        let mut pair = (a, b);
        pair.sort_selection();
        Self {
            a: pair.0,
            b: pair.1,
        }
    }

    pub fn other(&self, denom: &T) -> T {
        if &self.a == denom {
            self.b.clone()
        } else {
            self.a.clone()
        }
    }
}
