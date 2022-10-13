use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    serde::Serialize,
    AccountId,
};
use std::collections::HashMap;

use crate::errors::*;

#[derive(BorshDeserialize, BorshSerialize, Clone, Default, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub id: usize,
    pub tokens: Vec<AccountId>,
    pub liquidity: Vec<u128>,
    pub shares: HashMap<AccountId, Vec<u128>>,
}

impl Pool {
    pub fn new(id: usize, token1: AccountId, token2: AccountId) -> Pool {
        Pool {
            id: id,
            tokens: vec![token1, token2],
            liquidity: vec![0, 0],
            shares: HashMap::new(),
        }
    }

    pub fn add_liquidity(&mut self, token: &AccountId, amount: u128) {
        let index = self.get_index(&token);
        let account_id = env::predecessor_account_id();
        let mut vec = match self.shares.get(&account_id) {
            Some(vec) => vec.clone(),
            _ => vec![0, 0],
        };
        vec[index] += amount;
        self.liquidity[index] += amount;
        self.shares.insert(account_id, vec);
    }

    pub fn remove_liquidity(&mut self, token: &AccountId, amount: u128) {
        let index = self.get_index(&token);
        let account_id = env::predecessor_account_id();
        let share = self.get_share(&account_id, &token);
        assert!(
            self.shares.get(&account_id).is_some(),
            "{}",
            YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL
        );
        assert!(share >= amount, "{}", YOU_WANT_TO_REMOVE_TOO_MUCH_LIQUIDITY);
        let mut vec = self.shares.get(&account_id).unwrap().clone();
        vec[index] -= amount;
        self.shares.insert(account_id, vec);
        self.liquidity[index] -= amount;
    }

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
}
