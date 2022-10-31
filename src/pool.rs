use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

use crate::{
    position::{sqrt_price_to_tick, tick_to_price, Position},
    tick::Tick,
};

pub struct SwapResult {
    pub amount_in: f64,
    pub new_liquidity: f64,
    pub new_sqrt_price: f64,
}

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

    pub fn get_return(&self, _token_in: &AccountId, _amount_in: u128) -> u128 {
        // TO DO
        0
    }

    pub fn get_expense(&self, token_out: &AccountId, amount_out: u128) -> SwapResult {
        if self.check_swap_within_tick_is_possible(token_out, amount_out) {
            println!("Swap within a single ticks");
            return self.get_expense_within_tick(token_out, amount_out);
        } else {
            println!("Swap crossing several ticks");
            return self.get_expense_crossing_several_ticks(token_out, amount_out);
        }
    }

    fn get_expense_within_tick(&self, token_out: &AccountId, amount_out: u128) -> SwapResult {
        let new_sqrt_price;
        let amount_in;
        if token_out == &self.token1 {
            let delta_sqrt_price = amount_out as f64 / self.liquidity;
            new_sqrt_price = self.sqrt_price + delta_sqrt_price;
            amount_in = (1.0 / new_sqrt_price - 1.0 / self.sqrt_price) * self.liquidity;
        } else {
            let delta_reversed_sqrt_price = (amount_out as f64) / self.liquidity;
            new_sqrt_price = self.sqrt_price / (delta_reversed_sqrt_price * self.sqrt_price + 1.0);
            amount_in = (new_sqrt_price - self.sqrt_price) * self.liquidity;
        }
        return SwapResult {
            amount_in: -amount_in,
            new_liquidity: self.liquidity,
            new_sqrt_price,
        };
    }

    fn calculate_liquidity_within_tick(&self, sqrt_price: f64) -> f64 {
        let mut liquidity = 0.0;
        for position in &self.positions {
            if position.sqrt_lower_bound_price <= sqrt_price
                && sqrt_price <= position.sqrt_upper_bound_price
            {
                liquidity += position.liquidity;
            }
        }
        liquidity
    }

    fn get_amount_within_tick(
        &self,
        tick: &mut i32,
        sqrt_price: &mut f64,
        token_out: &AccountId,
        remaining: &mut f64,
    ) -> f64 {
        let liquidity = self.calculate_liquidity_within_tick(*sqrt_price);
        if token_out == &self.token1 {
            *tick += 1;
            let mut new_sqrt_price = tick_to_price(*tick);
            let mut amount_in = (1.0 / new_sqrt_price - 1.0 / self.sqrt_price) * liquidity;
            let amount_out = (new_sqrt_price - *sqrt_price) * liquidity;
            *sqrt_price = new_sqrt_price;
            if amount_out > *remaining {
                let delta_sqrt_price = *remaining / liquidity;
                new_sqrt_price = self.sqrt_price + delta_sqrt_price;
                amount_in = (1.0 / new_sqrt_price - 1.0 / self.sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining -= amount_out;
            }
            return -amount_in;
        } else {
            *tick -= 1;
            let mut new_sqrt_price = tick_to_price(*tick);
            let mut amount_in = (new_sqrt_price - *sqrt_price) * liquidity;
            let amount_out = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            *sqrt_price = new_sqrt_price;
            *remaining -= amount_out;
            if amount_out > *remaining {
                let delta_reversed_sqrt_price = *remaining / liquidity;
                new_sqrt_price =
                    self.sqrt_price / (delta_reversed_sqrt_price * self.sqrt_price + 1.0);
                amount_in = (new_sqrt_price - self.sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining -= amount_out;
            }
            return -amount_in;
        }
    }

    fn get_expense_crossing_several_ticks(
        &self,
        token_out: &AccountId,
        amount_out: u128,
    ) -> SwapResult {
        let mut collected = 0.0;
        let mut tick = self.tick;
        let mut price = self.sqrt_price;
        let mut remaining = amount_out as f64;
        while remaining > 0.0 {
            collected +=
                self.get_amount_within_tick(&mut tick, &mut price, token_out, &mut remaining)
        }
        let liquidity = self.calculate_liquidity_within_tick(price);
        SwapResult {
            amount_in: collected,
            new_liquidity: liquidity,
            new_sqrt_price: price,
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
        println!(
            "old price = {} new price = {}",
            self.sqrt_price, new_sqrt_price
        );
        println!("old tick = {} new_tick = {}", self.tick, new_tick);
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

    pub fn swap(&mut self, swap_result: SwapResult) {
        self.liquidity = swap_result.new_liquidity;
        self.sqrt_price = swap_result.new_sqrt_price;
    }
}
