use balance::AccountsInfo;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::{near_bindgen, BorshStorageKey};
use near_sdk::{AccountId, PanicOnDefault};
use pool::Pool;

mod balance;
mod pool;
mod token_receiver;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    Balance,
    Whitelist,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Account of the owner.
    owner_id: AccountId,
    /// List of all the pools.
    pools: Vec<Pool>,
    //  Accounts registered, keeping track all the amounts deposited, storage and more.
    accounts: AccountsInfo,
    //  Set of whitelisted tokens by "owner".
    whitelisted_tokens: UnorderedSet<AccountId>,
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
                accounts_info: UnorderedMap::new(StorageKey::Accounts),
            },
            whitelisted_tokens: UnorderedSet::new(StorageKey::Whitelist),
        }
    }

    #[private]
    pub fn create_pool(&mut self, token1: AccountId, token2: AccountId) -> u8 {
        let pool = Pool {
            id: self.pools.len() as u8,
            tokens: vec![token1, token2],
            liquidity: vec![0, 0],
        };
        let result = pool.id;
        self.pools.push(pool);
        return result;
    }

    pub fn get_pools(&self, from_index: u8, limit: u8) -> Vec<Pool> {
        let len = self.pools.len();
        let from = from_index as usize;
        if from > len {
            return vec![];
        }
        let to = (from_index + limit) as usize;
        let slice = match to <= len {
            true => &self.pools[from..to],
            _ => &self.pools[from..len],
        };
        slice.to_vec()
    }

    pub fn get_pool(&self, pool_id: u8) -> Option<&Pool> {
        for pool in &self.pools {
            if pool_id == pool.id {
                return Some(pool);
            }
        }
        None
    }

    pub fn get_balance(&self, account_id: &AccountId, token_id: &AccountId) -> Option<u128> {
        match self.accounts.get_balance(account_id) {
            None => return Some(0),
            Some(balance) => {
                return balance.balance.get(token_id);
            }
        }
    }

    pub fn add_liquidity(_pool_id: u8, _token: AccountId, _from: u8, _to: u8) {}

    pub fn remove_liquidity(_pool_id: u8, _token: AccountId) {}

    pub fn get_return(_pool_id: u8, _token1: AccountId, _amount: u8, _token2: AccountId) {}

    pub fn swap(_pool_id: u8, _token: String, _amount: u8) {}
}

// TO DO - Storage Management
