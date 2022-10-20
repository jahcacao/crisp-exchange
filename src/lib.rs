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
        self.pools.push(Pool::new(token1, token2));
        return self.pools.len() - 1;
    }

    pub fn get_pools(&self) -> Vec<Pool> {
        self.pools.clone()
    }

    pub fn get_pool(&self, pool_id: usize) -> Option<&Pool> {
        if pool_id >= self.pools.len() {
            None
        } else {
            Some(&self.pools[pool_id])
        }
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

    pub fn get_return(&self, pool_id: usize, token_in: &AccountId, amount_in: u128) -> u128 {
        assert!(pool_id < self.pools.len());
        let pool = &self.pools[pool_id];
        pool.get_return(token_in, amount_in)
    }

    pub fn get_price(&self, pool_id: usize) -> f64 {
        assert!(pool_id < self.pools.len());
        let pool = &self.pools[pool_id];
        pool.get_price()
    }

    pub fn swap(
        &mut self,
        pool_id: usize,
        token_in: AccountId,
        amount: u128,
        token_out: AccountId,
    ) {
        assert!(pool_id < self.pools.len());
        let pool = &mut self.pools[pool_id];
        pool.refresh_pool();
        let account_id = env::predecessor_account_id();
        self.accounts
            .decrease_balance(&account_id, &token_in, amount);
        let amount_out = pool.get_return(&token_in, amount);
        self.accounts
            .increase_balance(&account_id, &token_out, amount_out);
        pool.swap(token_in, token_out.to_string(), amount, amount_out);
    }

    pub fn open_position(
        &mut self,
        pool_id: usize,
        token0_liquidity: u128,
        token1_liquidity: u128,
        lower_bound_price: u128,
        upper_bound_price: u128,
    ) {
        assert!(pool_id < self.pools.len());
        let pool = &mut self.pools[pool_id];
        pool.open_position(
            token0_liquidity,
            token1_liquidity,
            lower_bound_price,
            upper_bound_price,
        );
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
