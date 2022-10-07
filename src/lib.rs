use balance::AccountsInfo;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, near_bindgen, BorshStorageKey};
use near_sdk::{AccountId, PanicOnDefault};
use pool::Pool;

mod balance;
mod pool;
mod token_receiver;

pub const GAS_FOR_FT_TRANSFER: u64 = 20_000_000_000_000;

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
    pub fn create_pool(&mut self, token1: AccountId, token2: AccountId) -> usize {
        self.pools
            .push(Pool::new(self.pools.len() as u8, token1, token2));
        return self.pools.len() - 1;
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

    pub fn withdraw(&mut self, token_id: &ValidAccountId, amount: u128) {
        let account_id = env::predecessor_account_id();
        if let Some(mut balance) = self.accounts.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token_id.to_string()) {
                assert!(amount <= current_amount, "Not enough tokens");
                balance
                    .balance
                    .insert(&token_id.to_string(), &(current_amount - amount));
                self.accounts.accounts_info.insert(&account_id, &balance);
                ext_fungible_token::ft_transfer(
                    account_id.to_string(),
                    U128(amount),
                    None,
                    token_id,
                    1,
                    GAS_FOR_FT_TRANSFER,
                );
                return;
            }
        }
        panic!("Token has not been deposited");
    }

    pub fn add_liquidity(&mut self, pool_id: u8, token_id: AccountId, amount: u128) {
        assert!(pool_id < self.pools.len() as u8, "Bad pool_id");
        let account_id = env::predecessor_account_id();
        if let Some(mut balance) = self.accounts.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token_id) {
                assert!(amount <= current_amount, "Not enough tokens");
                balance
                    .balance
                    .insert(&token_id, &(current_amount - amount));
                self.accounts.accounts_info.insert(&account_id, &balance);
                return;
            }
        }
        self.pools[pool_id as usize].add_liquidity(token_id, amount);
    }

    pub fn remove_liquidity(_pool_id: u8, _token: AccountId) {}

    pub fn get_return(_pool_id: u8, _token1: AccountId, _amount: u8, _token2: AccountId) {}

    pub fn swap(_pool_id: u8, _token: String, _amount: u8) {}
}

// TO DO - Storage Management
