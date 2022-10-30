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

    pub fn get_expense(&self, token_out: &AccountId, amount_out: u128) -> f64 {
        if self.check_swap_within_tick_is_possible(token_out, amount_out) {
            // whithin a single tick
            return self.get_expense_within_tick(token_out, amount_out);
        } else { // crossing several ticks
        }

        0.0
    }

    pub fn get_expense_within_tick(&self, token_out: &AccountId, amount_out: u128) -> f64 {
        if token_out == &self.token0 {
            let delta_sqrt_price = amount_out as f64 / self.liquidity;
            let new_sqrt_price = self.sqrt_price + delta_sqrt_price;
            return (1.0 / new_sqrt_price - 1.0 / self.sqrt_price) * self.liquidity;
        } else {
            let delta_reversed_sqrt_price = (amount_out as f64) / self.liquidity;
            let new_sqrt_price =
                self.sqrt_price / (delta_reversed_sqrt_price * self.sqrt_price + 1.0);
            return (new_sqrt_price - self.sqrt_price) * self.liquidity;
        }
    }

    pub fn check_swap_within_tick_is_possible(
        &self,
        token_out: &AccountId,
        amount_out: u128,
    ) -> bool {
        let new_sqrt_price;
        if token_out == &self.token0 {
            let delta_sqrt_price = amount_out as f64 / self.liquidity;
            new_sqrt_price = self.sqrt_price + delta_sqrt_price;
        } else {
            let delta_reversed_sqrt_price = (amount_out as f64) / self.liquidity;
            new_sqrt_price = self.sqrt_price / (delta_reversed_sqrt_price * self.sqrt_price + 1.0);
        }
        let new_tick = sqrt_price_to_tick(new_sqrt_price);
        if new_tick == self.tick {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_sqrt_price(&self) -> f64 {
        self.sqrt_price
    }

    pub fn refresh_liquidity(&mut self) {
        self.liquidity = 0.0;
        for position in &self.positions {
            if position.sqrt_lower_bound_price <= self.sqrt_price
                && position.sqrt_upper_bound_price >= self.sqrt_price
            {
                self.liquidity += position.liquidity;
            }
        }
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

    pub fn close_position(&mut self, id: usize) {
        let position = &self.positions[id];
        if position.sqrt_lower_bound_price <= self.sqrt_price
            && position.sqrt_upper_bound_price >= self.sqrt_price
        {
            self.liquidity -= position.liquidity;
        }
        self.positions.remove(id);
    }

    pub fn swap(&mut self, token_in: AccountId, amount_in: u128, amount_out: u128) {}
}
