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
    pub token0_liquidity: u128,
    pub token1_liquidity: u128,
    pub positions: Vec<Position>,
    pub sqrt_price: f64,
}

impl Pool {
    pub fn new(token0: AccountId, token1: AccountId, price: f64) -> Pool {
        Pool {
            token0,
            token1,
            token0_liquidity: 0,
            token1_liquidity: 0,
            positions: vec![],
            sqrt_price: price.sqrt(),
        }
    }

    pub fn refresh_price(&mut self) {
        if self.token0_liquidity != 0 && self.token1_liquidity != 0 {
            self.sqrt_price = (self.token1_liquidity as f64 / self.token0_liquidity as f64).sqrt();
        }
    }

    pub fn refresh_liquidity(&mut self) {
        self.token0_liquidity = 0;
        self.token1_liquidity = 0;
        println!("self.sqrt_price = {}", self.sqrt_price * self.sqrt_price);
        for position in &mut self.positions {
            position.refresh(self.sqrt_price);
            println!("position.token0_real_liquidity = {}", position.token0_real_liquidity);
            println!("position.token1_real_liquidity = {}", position.token1_real_liquidity);
            self.token0_liquidity += position.token0_real_liquidity;
            self.token1_liquidity += position.token1_real_liquidity;
        }
        self.refresh_price();
        println!("new self.sqrt_price = {}", self.sqrt_price * self.sqrt_price);
    }

    pub fn get_return(&self, token_in: &AccountId, amount_in: u128) -> u128 {
        if token_in == &self.token0 {
            return amount_in * self.token1_liquidity / (self.token0_liquidity + amount_in);
        } else if token_in == &self.token1 {
            return amount_in * self.token0_liquidity / (self.token1_liquidity + amount_in);
        } else {
            panic!("{}", BAD_TOKEN);
        }
    }

    pub fn get_price(&self) -> f64 {
        self.token1_liquidity as f64 / self.token0_liquidity as f64
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
        self.refresh_liquidity();
    }

    pub fn swap(&mut self, token_in: AccountId, amount_in: u128, amount_out: u128) {
        if token_in == self.token0 {
            self.token0_liquidity += amount_in;
            self.token1_liquidity += amount_out;
        } else {
            self.token0_liquidity += amount_out;
            self.token1_liquidity += amount_in;
        }
    }
}
