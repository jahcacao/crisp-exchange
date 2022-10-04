use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::AccountId;
use near_sdk::{near_bindgen, BorshStorageKey};

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    Whitelist,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Account of the owner.
    owner_id: AccountId,
    /// List of all the pools.
    pools: Vec<u8>,
    //  Accounts registered, keeping track all the amounts deposited, storage and more.
    accounts: LookupMap<AccountId, u8>,
    //  Set of whitelisted tokens by "owner".
    whitelisted_tokens: UnorderedSet<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id: owner_id.clone(),
            pools: Vec::new(),
            accounts: LookupMap::new(StorageKey::Accounts),
            whitelisted_tokens: UnorderedSet::new(StorageKey::Whitelist),
        }
    }

    pub fn create_pool() {}

    pub fn get_pools(from_index: u8, limit: u8) {}

    pub fn get_pool() {}

    pub fn storage_deposit() {}

    pub fn storage_balance_of(account_id: AccountId) {}

    pub fn get_balance(account_id: AccountId) {}

    pub fn add_liquidity(pool_id: u8, token: AccountId, from: u8, to: u8) {}

    pub fn remove_liquidity(pool_id: u8, token: AccountId) {}

    pub fn get_return(pool_id: u8, token1: AccountId, amount: u8, token2: AccountId) {}

    pub fn swap(pool_id: u8, token: String, amount: u8) {}
}
