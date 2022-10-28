use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

use crate::{
    position::{sqrt_price_to_tick, Position},
    tick::Tick,
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub token0: AccountId,
    pub token1: AccountId,
    pub liquidity: f64,
    pub sqrt_price: f64,
    pub tick: i32,
    pub ticks_range: HashMap<i32, Tick>,
    pub positions: Vec<Position>,
}

impl Pool {
    pub fn new(token0: AccountId, token1: AccountId, price: f64) -> Pool {
        let tick = sqrt_price_to_tick(price.sqrt());
        Pool {
            token0,
            token1,
            liquidity: 0.0,
            sqrt_price: price.sqrt(),
            positions: vec![],
            tick: tick,
            ticks_range: HashMap::new(),
        }
    }

    pub fn get_return(&self, token_in: &AccountId, amount_in: u128) -> u128 {
        // TO DO
        0
    }

    pub fn get_sqrt_price(&self) -> f64 {
        self.sqrt_price
    }

    pub fn open_position(&mut self, position: Position) {
        if position.sqrt_lower_bound_price <= self.sqrt_price
            && position.sqrt_upper_bound_price >= self.sqrt_price
        {
            self.liquidity += position.liquidity;
        }
        self.positions.push(position);
        // do something with tick range?
        // TO DO
    }

    pub fn swap(&mut self, token_in: AccountId, amount_in: u128, amount_out: u128) {}
}
