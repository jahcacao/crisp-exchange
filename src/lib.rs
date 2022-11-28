use balance::AccountsInfo;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen};
use near_sdk::{AccountId, PanicOnDefault};
use nft::metadata::{NFTContractMetadata, Token, TokenId, TokenMetadata};
use pool::Pool;

use crate::errors::*;
use crate::position::Position;

mod balance;
mod errors;
pub mod pool;
mod position;
mod token_receiver;

use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, CryptoHash, Promise, PromiseOrValue};
use std::collections::HashMap;

mod nft;

#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
}

pub const NFT_METADATA_SPEC: &str = "1.0.0";
pub const NFT_STANDARD_NAME: &str = "nep171";
pub const BASIS_POINT: f64 = 1.0001;
pub const BASIS_POINT_TO_PERCENT: f64 = 10000.0;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub pools: Vec<Pool>,
    //  Accounts registered, keeping track all the amounts deposited
    pub accounts: AccountsInfo,
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub tokens_by_id: LookupMap<TokenId, Token>,
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,
    pub metadata: LazyOption<NFTContractMetadata>,
    pub positions_opened: u128,
}

#[near_bindgen]
impl Contract {
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
            owner_id,
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
        self.pools.len() - 1
    }

    #[private]
    pub fn remove_pool(&mut self, pool_id: usize) {
        self.assert_pool_exists(pool_id);
        self.pools.remove(pool_id);
    }

    pub fn get_pools(&self) -> Vec<Pool> {
        self.pools.clone()
    }

    fn assert_pool_exists(&self, pool_id: usize) {
        assert!(pool_id < self.pools.len(), "{}", BAD_POOL_ID);
    }

    fn assert_account_owns_nft(account_id: &AccountId, nft_owner: &AccountId) {
        assert!(account_id == nft_owner);
    }

    pub fn get_pool(&self, pool_id: usize) -> Pool {
        self.assert_pool_exists(pool_id);
        self.pools[pool_id].clone()
    }

    pub fn get_balance(&self, account_id: &AccountId, token: &AccountId) -> U128 {
        let balance = match self.accounts.get_balance(account_id) {
            None => Some(0),
            Some(balance) => balance.get(token),
        };
        if let Some(amount) = balance {
            amount.into()
        } else {
            U128(0)
        }
    }

    pub fn get_balance_all_tokens(&self, account_id: &AccountId) -> String {
        if let Some(balance) = self.accounts.get_balance(account_id) {
            let mut result = String::new();
            for (token, amount) in balance.iter() {
                result += &format!("{token}: {amount}, ");
            }
            result
        } else {
            String::new()
        }
    }

    pub fn withdraw(&mut self, token: AccountId, amount: U128) {
        let account_id = env::predecessor_account_id();
        let amount: u128 = amount.into();
        self.accounts.withdraw(&account_id, &token, amount);
    }

    pub fn get_return(&self, pool_id: usize, token_in: &AccountId, amount_in: U128) -> U128 {
        let pool = self.get_pool(pool_id);
        let amount_in: u128 = amount_in.into();
        let swap_result = pool.get_swap_result(token_in, amount_in, pool::SwapDirection::Return);
        (swap_result.amount.round() as u128).into()
    }

    pub fn get_expense(&self, pool_id: usize, token_out: &AccountId, amount_out: U128) -> U128 {
        let pool = self.get_pool(pool_id);
        let amount_out: u128 = amount_out.into();
        let swap_result = pool.get_swap_result(token_out, amount_out, pool::SwapDirection::Expense);
        (swap_result.amount.round() as u128).into()
    }

    pub fn get_price(&self, pool_id: usize) -> f64 {
        let pool = self.get_pool(pool_id);
        let sqrt_price = pool.get_sqrt_price();
        sqrt_price * sqrt_price
    }

    pub fn swap(
        &mut self,
        pool_id: usize,
        token_in: AccountId,
        amount_in: U128,
        token_out: AccountId,
    ) -> U128 {
        self.assert_pool_exists(pool_id);
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let amount_in: u128 = amount_in.into();
        self.accounts
            .decrease_balance(&account_id, &token_in, amount_in);
        let swap_result = pool.get_swap_result(&token_in, amount_in, pool::SwapDirection::Return);
        self.accounts
            .apply_collected_fees(&swap_result.collected_fees, &token_out);
        self.accounts
            .increase_balance(&account_id, &token_out, swap_result.amount.round() as u128);
        let fees_amount = swap_result.amount * (pool.protocol_fee as f64 + pool.rewards as f64)
            / BASIS_POINT_TO_PERCENT;
        self.accounts
            .decrease_balance(&account_id, &token_out, fees_amount.round() as u128);
        pool.apply_swap_result(&swap_result);
        let current_timestamp = env::block_timestamp();
        pool.refresh(current_timestamp);
        (swap_result.amount.round() as u128).into()
    }

    pub fn open_position(
        &mut self,
        pool_id: usize,
        token0_liquidity: Option<U128>,
        token1_liquidity: Option<U128>,
        lower_bound_price: f64,
        upper_bound_price: f64,
    ) -> u128 {
        self.assert_pool_exists(pool_id);
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
            position.token0_locked.round() as u128,
        );
        self.accounts.decrease_balance(
            &account_id,
            &pool.token1,
            position.token1_locked.round() as u128,
        );
        pool.open_position(position.clone());
        let current_timestamp = env::block_timestamp();
        pool.refresh(current_timestamp);
        let metadata = TokenMetadata::new(pool_id, &position);
        self.nft_mint(id.to_string(), account_id.clone(), metadata);
        id
    }

    pub fn close_position(&mut self, pool_id: usize, id: u128) -> bool {
        self.assert_pool_exists(pool_id);
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let current_timestamp = env::block_timestamp();
        let token = self.tokens_by_id.get(&id.to_string()).unwrap();
        Self::assert_account_owns_nft(&account_id, &token.owner_id);
        for (i, position) in &mut pool.positions.iter().enumerate() {
            if position.id == id {
                let token0 = &pool.token0;
                let token1 = &pool.token1;
                let position = position.clone();
                let amount0 = position.token0_locked.round() as u128;
                let amount1 = position.token1_locked.round() as u128;
                self.accounts.increase_balance(&account_id, token0, amount0);
                self.accounts.increase_balance(&account_id, token1, amount1);
                pool.close_position(i);
                pool.refresh(current_timestamp);
                return true;
            }
        }
        false
    }
}
