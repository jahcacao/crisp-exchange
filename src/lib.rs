use balance::AccountsInfo;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, PanicOnDefault};
use pool::Pool;

use crate::errors::*;

mod balance;
mod errors;
mod fees;
mod pool;
mod position;
mod token_receiver;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Account of the owner.
    owner_id: AccountId,
    /// List of all the pools.
    pools: Vec<Pool>,
    //  Accounts registered, keeping track all the amounts deposited, storage and more.
    accounts: AccountsInfo,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id: owner_id.clone(),
            pools: Vec::new(),
            accounts: AccountsInfo {
                accounts_info: UnorderedMap::new(b"a"),
            },
        }
    }

    #[private]
    pub fn create_pool(&mut self, token1: AccountId, token2: AccountId) -> usize {
        self.pools.push(Pool::new(self.pools.len(), token1, token2));
        return self.pools.len() - 1;
    }

    pub fn get_pools(&self) -> Vec<Pool> {
        self.pools.clone()
    }

    pub fn get_pool(&self, pool_id: usize) -> Option<&Pool> {
        for pool in &self.pools {
            if pool_id == pool.id {
                return Some(pool);
            }
        }
        None
    }

    pub fn get_balance(&self, account_id: &AccountId, token: &AccountId) -> Option<u128> {
        match self.accounts.get_balance(account_id) {
            None => return Some(0),
            Some(balance) => {
                return balance.get(token);
            }
        }
    }

    pub fn get_balance_all_tokens(&self, account: &AccountId) -> String {
        if let Some(balance) = self.accounts.get_balance(account) {
            let mut result = String::new();
            for (token, amount) in balance.iter() {
                result += &format!("{token}: {amount}, ");
            }
            return result;
        } else {
            return String::new();
        }
    }

    pub fn withdraw(&mut self, token: AccountId, amount: u128) {
        let account_id = env::predecessor_account_id();
        self.accounts.withdraw(account_id, token, amount);
    }

    // pub fn add_liquidity(&mut self, pool_id: u8, token: AccountId, amount: u128) {
    //     assert!(pool_id < self.pools.len() as u8, "{}", BAD_POOL_ID);
    //     let account_id = env::predecessor_account_id();
    //     self.accounts.decrease_balance(&account_id, &token, amount);
    //     self.pools[pool_id as usize].add_liquidity(&account_id, &token, amount);
    // }

    // pub fn remove_liquidity(&mut self, pool_id: u8, token: AccountId, amount: u128) {
    //     assert!(pool_id < self.pools.len() as u8, "{}", BAD_POOL_ID);
    //     let account_id = env::predecessor_account_id();
    //     self.accounts.increase_balance(&account_id, &token, amount);
    //     self.pools[pool_id as usize].remove_liquidity(&account_id, &token, amount);
    // }

    pub fn get_return(&self, pool_id: usize, token_in: &AccountId, amount_in: u128) -> u128 {
        assert!(pool_id < self.pools.len());
        let pool = &self.pools[pool_id];
        pool.get_return(token_in, amount_in)
    }

    pub fn get_price(&self, pool_id: usize, token_in: &AccountId) -> u128 {
        self.get_return(pool_id, token_in, 1)
    }

    // pub fn swap(&mut self, pool_id: usize, token_in: AccountId, amount: u128) {
    //     assert!(pool_id < self.pools.len());
    //     let account_id = env::predecessor_account_id();
    //     let other_index = self.pools[pool_id].get_other_index(&token_in);
    //     self.accounts
    //         .decrease_balance(&account_id, &token_in, amount);
    //     let amount_out = self.get_return(pool_id, &token_in, amount);
    //     let pool = &mut self.pools[pool_id];
    //     let token_out = &pool.tokens[other_index].clone();
    //     self.accounts
    //         .increase_balance(&account_id, &token_out, amount_out);
    //     pool.add_liquidity(&account_id, &token_in, amount);
    //     pool.remove_liquidity(&account_id, token_out, amount_out);
    // }

    pub fn open_position(
        &mut self,
        pool_id: usize,
        token_amount_1: (AccountId, u128),
        token_amount_2: (AccountId, u128),
        lower_price: u128,
        upper_price: u128,
    ) {
        assert!(pool_id < self.pools.len());
        let pool = &mut self.pools[pool_id];
        pool.open_position(token_amount_1, token_amount_2, lower_price, upper_price);
    }

    pub fn close_position(&mut self) {}
}

// open order for concentrated liquidity
// fees
// rewards
// test get_balance
// test get_balance_all_tokens
// test withdraw
// test deposits
// test liquidity
// test get return
// tests for swap
// tests for fees
// tests for rewards
// front
