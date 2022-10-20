use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

use crate::{errors::*, position::Position};

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub token0: AccountId,
    pub token1: AccountId,
    pub token_0_active_liquidity: u128,
    pub token_1_active_liquidity: u128,
    pub positions: Vec<Position>,
    pub sqrt_price: f64,
}

impl Pool {
    pub fn new(token0: AccountId, token1: AccountId) -> Pool {
        Pool {
            token0,
            token1,
            token_0_active_liquidity: 0,
            token_1_active_liquidity: 0,
            positions: vec![],
            sqrt_price: 1.0,
        }
    }

    pub fn refresh_price(&mut self) {
        self.sqrt_price =
            (self.token_1_active_liquidity as f64 / self.token_0_active_liquidity as f64).sqrt();
    }

    pub fn refresh_pool(&mut self) {
        loop {
            let old_price = self.sqrt_price;
            self.refresh_price();
            self.token_0_active_liquidity = 0;
            self.token_1_active_liquidity = 0;
            for position in &mut self.positions {
                position.refresh(self.sqrt_price);
                self.token_0_active_liquidity += position.token0_real_liquidity;
                self.token_1_active_liquidity += position.token1_real_liquidity;
            }
            self.refresh_price();
            if old_price == self.sqrt_price {
                return;
            }
        }
    }

    pub fn get_return(&self, token_in: &AccountId, amount_in: u128) -> u128 {
        if token_in == &self.token0 {
            return amount_in * self.token_1_active_liquidity
                / (self.token_0_active_liquidity + amount_in);
        } else if token_in == &self.token1 {
            return amount_in * self.token_0_active_liquidity
                / (self.token_1_active_liquidity + amount_in);
        } else {
            panic!("{}", BAD_TOKEN);
        }
    }

    pub fn get_price(&self) -> f64 {
        self.token_1_active_liquidity as f64 / self.token_0_active_liquidity as f64
    }

    pub fn open_position(
        &mut self,
        token0_liquidity: u128,
        token1_liquidity: u128,
        lower_bound_price: u128,
        upper_bound_price: u128,
    ) {
        let position = Position::new(
            token0_liquidity,
            token1_liquidity,
            lower_bound_price,
            upper_bound_price,
        );
        self.positions.push(position);
    }

    pub fn swap(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
        amount_out: u128,
    ) {
        if token_in == self.token0 {
            self.token_0_active_liquidity += amount_in;
            self.token_1_active_liquidity += amount_out;
        } else {
            self.token_0_active_liquidity += amount_out;
            self.token_1_active_liquidity += amount_in;
        }
    }
}
