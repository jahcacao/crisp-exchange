use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};
use std::collections::HashMap;

use crate::{errors::*, position::Position};

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub id: usize,
    pub tokens: Vec<AccountId>,
    pub liquidity: Vec<u128>,
    pub positions: Vec<Position>,
    pub shares: HashMap<AccountId, Vec<u128>>,
    pub price: u128,
}

impl Pool {
    pub fn new(id: usize, token1: AccountId, token2: AccountId) -> Pool {
        Pool {
            id: id,
            tokens: vec![token1, token2],
            liquidity: vec![0, 0],
            positions: vec![],
            shares: HashMap::new(),
            price: 0,
        }
    }

    // pub fn add_liquidity(&mut self, account_id: &AccountId, token: &AccountId, amount: u128) {
    //     let index = self.get_index(&token);
    //     let mut vec = match self.shares.get(account_id) {
    //         Some(vec) => vec.clone(),
    //         _ => vec![0, 0],
    //     };
    //     vec[index] += amount;
    //     self.liquidity[index] += amount;
    //     self.shares.insert(account_id.to_string(), vec);
    // }

    // pub fn remove_liquidity(&mut self, account_id: &AccountId, token: &AccountId, amount: u128) {
    //     let index = self.get_index(&token);
    //     let share = self.get_share(&account_id, &token);
    //     assert!(
    //         self.shares.get(account_id).is_some(),
    //         "{}",
    //         YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL
    //     );
    //     assert!(share >= amount, "{}", YOU_WANT_TO_REMOVE_TOO_MUCH_LIQUIDITY);
    //     let mut vec = self.shares.get(account_id).unwrap().clone();
    //     vec[index] -= amount;
    //     self.shares.insert(account_id.to_string(), vec);
    //     self.liquidity[index] -= amount;
    // }

    pub fn get_share(&self, account_id: &AccountId, token: &AccountId) -> u128 {
        let index = self.get_index(token);
        let vec = match self.shares.get(account_id) {
            Some(vec) => vec.clone(),
            _ => vec![0, 0],
        };
        vec[index]
    }

    pub fn get_index(&self, token: &AccountId) -> usize {
        if token == &self.tokens[0] {
            0 as usize
        } else if token == &self.tokens[1] {
            1 as usize
        } else {
            panic!("{}", BAD_TOKEN);
        }
    }

    pub fn get_other_index(&self, token: &AccountId) -> usize {
        if token == &self.tokens[0] {
            1 as usize
        } else if token == &self.tokens[1] {
            0 as usize
        } else {
            panic!("{}", BAD_TOKEN);
        }
    }

    pub fn get_return(&self, token_in: &AccountId, amount_in: u128) -> u128 {
        let index_in = self.get_index(&token_in);
        let index_out = self.get_other_index(&token_in);
        let amount_out: u128 =
            (self.liquidity[index_out] * amount_in) / (self.liquidity[index_in] + amount_in);
        amount_out
    }

    pub fn get_price(&self, token_in: &AccountId) -> u128 {
        self.get_return(token_in, 1)
    }

    pub fn open_position(
        &mut self,
        token_amount_1: (AccountId, u128),
        token_amount_2: (AccountId, u128),
        lower_price: u128,
        upper_price: u128,
    ) {
        assert!(lower_price < upper_price);
        let mut position = Position {
            primary_token: token_amount_1.0.to_string(),
            lower_price,
            upper_price,
            liquidity: vec![token_amount_1.1, token_amount_2.1],
            is_active: false,
        };
        let price = self.get_price(&token_amount_1.0);
        if price > upper_price {
            assert!(token_amount_1.1 == 0);
        } else if price < lower_price {
            assert!(token_amount_2.1 == 0);
        } else {
            position.is_active = true;
        }
        self.positions.push(position);
    }

    pub fn refresh_liquidity(&mut self) {
        self.liquidity[0] = 0;
        self.liquidity[1] = 0;
        for position in &self.positions {
            if position.is_active {
                self.liquidity[0] += position.liquidity[0];
                self.liquidity[1] += position.liquidity[1];
            }
        }
    }

    pub fn refresh_positions(&mut self) {
        let price1 = self.get_price(&self.tokens[0]);
        let price2 = self.get_price(&self.tokens[1]);
        for position in &mut self.positions {
            if self.tokens[0] == position.primary_token {
                position.refresh_activity(price1);
            } else {
                position.refresh_activity(price2);
            }
        }
    }
}
