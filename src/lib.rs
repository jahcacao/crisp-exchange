use balance::AccountsInfo;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, PanicOnDefault};
use pool::Pool;

use crate::errors::*;
use crate::position::Position;

mod balance;
mod errors;
mod pool;
mod position;
mod token_receiver;

use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, CryptoHash, Promise, PromiseOrValue};
use std::collections::HashMap;

pub use crate::approval::*;
pub use crate::events::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::royalty::*;

mod approval;
mod enumeration;
mod events;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod royalty;
mod tests;

#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
}

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "1.0.0";
/// This is the name of the NFT standard we're using
pub const NFT_STANDARD_NAME: &str = "nep171";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Account of the owner.
    pub owner_id: AccountId,
    /// List of all the pools.
    pub pools: Vec<Pool>,
    //  Accounts registered, keeping track all the amounts deposited, storage and more.
    pub accounts: AccountsInfo,
    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
    //keeps track of the token struct for a given token ID
    pub tokens_by_id: LookupMap<TokenId, Token>,
    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,
    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,
    // number of positions opened
    pub positions_opened: u128,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        let metadata = NFTContractMetadata {
            spec: "nft-1.0.0".to_string(),
            name: "Crisp Exchange Contract".to_string(),
            symbol: "CRISP.EX".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        };
        Self {
            owner_id: owner_id.clone(),
            pools: Vec::new(),
            accounts: AccountsInfo {
                accounts_info: UnorderedMap::new(b"a"),
            },
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            positions_opened: 0,
        }
    }

    #[private]
    pub fn create_pool(
        &mut self,
        token1: AccountId,
        token2: AccountId,
        initial_price: f64,
        protocol_fee: u16,
        rewards: u16,
    ) -> usize {
        self.pools.push(Pool::new(
            token1,
            token2,
            initial_price,
            protocol_fee,
            rewards,
        ));
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

    pub fn get_balance_all_tokens(&self, account_id: &AccountId) -> String {
        if let Some(balance) = self.accounts.get_balance(account_id) {
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

    pub fn get_return(&self, pool_id: usize, token_in: &AccountId, amount_in: u128) -> f64 {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
        let pool = &self.pools[pool_id];
        let swap_result = pool.get_swap_result(token_in, amount_in, pool::SwapDirection::Return);
        swap_result.amount
    }

    pub fn get_expense(&self, pool_id: usize, token_out: &AccountId, amount_out: u128) -> f64 {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
        let pool = &self.pools[pool_id];
        let swap_result = pool.get_swap_result(token_out, amount_out, pool::SwapDirection::Expense);
        swap_result.amount
    }

    pub fn get_price(&self, pool_id: usize) -> f64 {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
        let pool = &self.pools[pool_id];
        let sqrt_price = pool.get_sqrt_price();
        sqrt_price * sqrt_price
    }

    pub fn swap_out(
        &mut self,
        pool_id: usize,
        token_in: AccountId,
        amount_out: u128,
        token_out: AccountId,
    ) -> u128 {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        self.accounts
            .increase_balance(&account_id, &token_out, amount_out);
        let swap_result =
            pool.get_swap_result(&token_out, amount_out, pool::SwapDirection::Expense);
        self.accounts
            .apply_collected_fees(&swap_result.collected_fees, &token_in);
        self.accounts
            .decrease_balance(&account_id, &token_in, swap_result.amount as u128);
        let fees_amount = (swap_result.amount as f64)
            * (pool.protocol_fee as f64 + pool.rewards as f64)
            / 10000.0;
        self.accounts
            .decrease_balance(&account_id, &token_in, fees_amount as u128);
        pool.apply_swap_result(&swap_result);
        swap_result.amount as u128
    }

    pub fn swap_in(
        &mut self,
        pool_id: usize,
        token_in: AccountId,
        amount_in: u128,
        token_out: AccountId,
    ) -> u128 {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        self.accounts
            .decrease_balance(&account_id, &token_in, amount_in);
        let swap_result = pool.get_swap_result(&token_out, amount_in, pool::SwapDirection::Return);
        self.accounts
            .apply_collected_fees(&swap_result.collected_fees, &token_out);
        self.accounts
            .increase_balance(&account_id, &token_out, swap_result.amount as u128);
        let fees_amount =
            swap_result.amount * (pool.protocol_fee as f64 + pool.rewards as f64) / 10000.0;
        self.accounts
            .decrease_balance(&account_id, &token_out, fees_amount as u128);
        pool.apply_swap_result(&swap_result);
        swap_result.amount as u128
    }

    pub fn open_position(
        &mut self,
        pool_id: usize,
        token0_liquidity: Option<u128>,
        token1_liquidity: Option<u128>,
        lower_bound_price: f64,
        upper_bound_price: f64,
    ) -> u128 {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
        let id = self.positions_opened;
        self.positions_opened += 1;
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let position = Position::new(
            id,
            account_id.clone(),
            token0_liquidity,
            token1_liquidity,
            lower_bound_price,
            upper_bound_price,
            pool.sqrt_price,
        );
        self.accounts.decrease_balance(
            &account_id,
            &pool.token0,
            position.token0_real_liquidity as u128,
        );
        self.accounts.decrease_balance(
            &account_id,
            &pool.token1,
            position.token1_real_liquidity as u128,
        );
        pool.open_position(position.clone());
        let metadata = TokenMetadata::new(pool_id, &position);
        self.nft_mint(id.to_string(), account_id.clone(), metadata);
        return id;
    }

    pub fn close_position(&mut self, pool_id: usize, id: u128) -> bool {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let token = self.tokens_by_id.get(&id.to_string()).unwrap();
        assert!(account_id == token.owner_id);
        for (i, position) in &mut pool.positions.iter().enumerate() {
            if position.id == id {
                let token0 = &pool.token0;
                let token1 = &pool.token1;
                let mut position = position.clone();
                position.refresh(pool.sqrt_price);
                let amount0 = position.token0_real_liquidity as u128;
                let amount1 = position.token1_real_liquidity as u128;
                self.accounts.increase_balance(&account_id, token0, amount0);
                self.accounts.increase_balance(&account_id, token1, amount1);
                pool.close_position(i);
                self.nft_burn(id.to_string());
                return true;
            }
        }
        return false;
    }
}

// front
// decimals