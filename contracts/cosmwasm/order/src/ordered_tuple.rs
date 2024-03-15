use cosmwasm_schema::cw_serde;
pub use tuples::*;

#[cw_serde]
pub struct OrderedTuple2<T> {
    pub a: T,
    pub b: T,
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

impl<T: PartialOrd + Clone + PartialOrd + PartialEq> From<(T, T)> for OrderedTuple2<T> {
    fn from(pair: (T, T)) -> Self {
        Self::new(pair.0, pair.1)
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
