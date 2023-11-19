

use crate::prelude::*;

use crate::OrderSubMsg;


pub type OrderId = Uint128;

#[cw_serde]
pub struct OrderItem {
    pub owner: Addr,
    pub msg: OrderSubMsg,
    pub given: Coin,
    pub order_id: OrderId,
}

impl OrderItem {
    pub fn fill(&mut self, transfer: &Coin) {
        self.msg.wants.amount -= transfer.amount;
        self.given.amount -= transfer.amount * self.given.amount / self.msg.wants.amount;
    }
}