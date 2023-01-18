use balance::borrow::{Borrow, BorrowId};
use balance::deposit::{Deposit, DepositId};
use balance::reserve::Reserve;
use balance::BalancesMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, ext_contract, near_bindgen};
use near_sdk::{AccountId, PanicOnDefault};
use nft::metadata::{NFTContractMetadata, Token, TokenId, TokenMetadata};
use pool::Pool;

pub use crate::balance::*;
use crate::errors::*;
use crate::nft::nft_core::NonFungibleTokenCore;
use crate::position::Position;

pub mod balance;
mod errors;
pub mod pool;
mod position;
mod token_receiver;

use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, CryptoHash, Promise, PromiseOrValue};
use std::collections::HashMap;

mod borrow;
mod deposit;
mod nft;
mod reserve;

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
pub const APR_DEPOSIT: u16 = 500;
pub const APR_BORROW: u16 = 1000;
pub const BORROW_RATIO: f64 = 0.8;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub pools: Vec<Pool>,
    //  Accounts registered, keeping track all the amounts deposited
    pub balances_map: BalancesMap,
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub tokens_by_id: LookupMap<TokenId, Token>,
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,
    pub metadata: LazyOption<NFTContractMetadata>,
    pub positions_opened: u128,
    pub deposits: HashMap<DepositId, Deposit>,
    pub deposits_created_number: DepositId,
    pub reserves: UnorderedMap<AccountId, Reserve>,
    pub borrows: UnorderedMap<BorrowId, Borrow>,
    pub borrows_number: BorrowId,
}

#[ext_contract(ext_self)]
pub trait SelfCallbacks {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
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
            balances_map: UnorderedMap::new(b"a"),
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
            deposits: HashMap::new(),
            deposits_created_number: 0,
            reserves: UnorderedMap::new(b"r"),
            borrows: UnorderedMap::new(b"b"),
            borrows_number: 0,
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
        let balance = match self.balances_map.get(account_id) {
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
        if let Some(balance) = self.balances_map.get(account_id) {
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
        self.balance_withdraw(&account_id, &token, amount);
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
        let account_id = env::predecessor_account_id();
        let amount_in: u128 = amount_in.into();
        self.decrease_balance(&account_id, &token_in, amount_in);
        let pool = &mut self.pools[pool_id];
        let swap_result = pool.get_swap_result(&token_in, amount_in, pool::SwapDirection::Return);
        self.apply_collected_fees(&swap_result.collected_fees, &token_out);
        self.increase_balance(&account_id, &token_out, swap_result.amount.round() as u128);
        let pool = &self.pools[pool_id];
        let fees_amount = swap_result.amount * (pool.protocol_fee as f64 + pool.rewards as f64)
            / BASIS_POINT_TO_PERCENT;
        self.decrease_balance(&account_id, &token_out, fees_amount.round() as u128);
        let pool = &mut self.pools[pool_id];
        pool.apply_swap_result(&swap_result);
        pool.refresh(env::block_timestamp());
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
        let position_id = self.positions_opened;
        self.positions_opened += 1;
        let pool = &self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let position = Position::new(
            account_id.clone(),
            token0_liquidity,
            token1_liquidity,
            lower_bound_price,
            upper_bound_price,
            pool.sqrt_price,
        );
        let token0 = pool.token0.clone();
        let token1 = pool.token1.clone();
        self.decrease_balance(&account_id, &token0, position.token0_locked.round() as u128);
        self.decrease_balance(&account_id, &token1, position.token1_locked.round() as u128);
        let pool = &mut self.pools[pool_id];
        pool.open_position(position_id, position.clone());
        pool.refresh(env::block_timestamp());
        let metadata = TokenMetadata::new(pool_id, position_id, &position);
        self.nft_mint(position_id.to_string(), account_id.clone(), metadata);
        position_id
    }

    pub fn close_position(&mut self, pool_id: usize, position_id: u128) {
        self.assert_pool_exists(pool_id);
        let pool = &self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let token = self.tokens_by_id.get(&position_id.to_string()).unwrap();
        Self::assert_account_owns_nft(&account_id, &token.owner_id);
        let position = pool.positions.get(&position_id).expect("Not found");
        let amount0 = position.token0_locked.round() as u128;
        let amount1 = position.token1_locked.round() as u128;
        let token0 = pool.token0.clone();
        let token1 = pool.token1.clone();
        self.increase_balance(&account_id, &token0, amount0);
        self.increase_balance(&account_id, &token1, amount1);
        let pool = &mut self.pools[pool_id];
        pool.close_position(position_id);
    }

    pub fn add_liquidity(
        &mut self,
        pool_id: usize,
        position_id: u128,
        token0_liquidity: Option<U128>,
        token1_liquidity: Option<U128>,
    ) {
        self.assert_pool_exists(pool_id);
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let token = self.tokens_by_id.get(&position_id.to_string()).unwrap();
        Self::assert_account_owns_nft(&account_id, &token.owner_id);
        let mut position = pool.positions.get(&position_id).expect("Not found").clone();
        let token0_locked_before = position.token0_locked as u128;
        let token1_locked_before = position.token1_locked as u128;
        position.add_liquidity(token0_liquidity, token1_liquidity, pool.sqrt_price);
        let token0_locked_after = position.token0_locked as u128;
        let token1_locked_after = position.token1_locked as u128;
        pool.positions.insert(position_id, position);
        pool.refresh(env::block_timestamp());
        let token0 = pool.token0.to_string();
        let token1 = pool.token1.to_string();
        self.decrease_balance(
            &account_id,
            &token0,
            token0_locked_after - token0_locked_before,
        );
        self.decrease_balance(
            &account_id,
            &token1,
            token1_locked_after - token1_locked_before,
        );
    }

    pub fn remove_liquidity(
        &mut self,
        pool_id: usize,
        position_id: u128,
        token0_liquidity: Option<U128>,
        token1_liquidity: Option<U128>,
    ) {
        self.assert_pool_exists(pool_id);
        let pool = &mut self.pools[pool_id];
        let account_id = env::predecessor_account_id();
        let token = self.tokens_by_id.get(&position_id.to_string()).unwrap();
        Self::assert_account_owns_nft(&account_id, &token.owner_id);
        let mut position = pool.positions.get(&position_id).expect("Not found").clone();
        let token0_locked_before = position.token0_locked as u128;
        let token1_locked_before = position.token1_locked as u128;
        position.remove_liquidity(token0_liquidity, token1_liquidity, pool.sqrt_price);
        let token0_locked_after = position.token0_locked as u128;
        let token1_locked_after = position.token1_locked as u128;
        pool.positions.insert(position_id, position);
        pool.refresh(env::block_timestamp());
        let token0 = pool.token0.to_string();
        let token1 = pool.token1.to_string();
        self.increase_balance(
            &account_id,
            &token0,
            token0_locked_before - token0_locked_after,
        );
        self.increase_balance(
            &account_id,
            &token1,
            token1_locked_before - token1_locked_after,
        );
    }

    #[private]
    pub fn create_reserve(&mut self, reserve_token: AccountId) {
        let reserve = Reserve::default();
        self.reserves.insert(&reserve_token, &reserve);
    }

    pub fn create_deposit(&mut self, asset: AccountId, amount: U128) {
        let account_id = env::predecessor_account_id();
        let timestamp = env::block_timestamp();
        let deposit = Deposit {
            owner_id: account_id.clone(),
            asset: asset.clone(),
            amount: amount.0,
            timestamp,
            last_update_timestamp: timestamp,
            apr: APR_DEPOSIT,
            growth: 0,
        };
        self.deposits.insert(self.deposits_created_number, deposit);
        self.deposits_created_number += 1;
        self.decrease_balance(&account_id, &asset.to_string(), amount.0);
        let mut reserve = self.reserves.get(&asset).unwrap();
        reserve.increase_deposit(amount.0);
        self.reserves.insert(&asset, &reserve);
    }

    pub fn close_deposit(&mut self, deposit_id: U128) {
        let account_id = env::predecessor_account_id();
        if let Some(deposit) = self.deposits.remove(&deposit_id.0) {
            assert_eq!(deposit.owner_id, account_id, "You do not own this deposit");
            self.increase_balance(&account_id, &deposit.asset, deposit.amount);
            self.increase_balance(&account_id, &deposit.asset, deposit.growth);
            let mut reserve = self.reserves.get(&deposit.asset).unwrap();
            reserve.decrease_deposit(deposit.amount);
            self.reserves.insert(&deposit.asset, &reserve);
        } else {
            panic!("Deposit not found");
        }
    }

    pub fn refresh_deposits_growth(&mut self) {
        let current_timestamp = env::block_timestamp();
        println!("current_timestamp = {current_timestamp}");
        for (_, deposit) in &mut self.deposits {
            deposit.refresh_growth(current_timestamp);
        }
    }

    #[allow(unused_assignments)]
    pub fn take_deposit_growth(&mut self, deposit_id: U128, amount: U128) -> U128 {
        let account_id = env::predecessor_account_id();
        let mut asset: Option<AccountId> = None;
        let mut growth = 0;
        if let Some(deposit) = self.deposits.get_mut(&deposit_id.0) {
            assert_eq!(deposit.owner_id, account_id, "You do not own this deposit");
            deposit.refresh_growth(env::block_timestamp());
            growth = deposit.take_growth(amount.0);
            asset = Some(deposit.asset.clone());
        } else {
            panic!("Deposit not found");
        }
        if let Some(asset) = asset {
            self.increase_balance(&account_id, &asset, growth);
            return growth.into();
        }
        0.into()
    }

    pub fn get_account_deposits(&self, account_id: AccountId) -> HashMap<DepositId, Deposit> {
        let mut result: HashMap<DepositId, Deposit> = HashMap::new();
        for (id, deposit) in &self.deposits {
            if deposit.owner_id == account_id {
                result.insert(*id, deposit.clone());
            }
        }
        result
    }

    pub fn supply_collateral_and_borrow_simple(
        &mut self,
        pool_id: usize,
        position_id: u128,
    ) -> U128 {
        let account_id = env::predecessor_account_id();
        let pool = &self.pools[pool_id];
        let position = pool
            .positions
            .get(&position_id)
            .expect("Position not found")
            .clone();
        let token = pool.token1.clone();
        let collateral = position.total_locked;
        let borrowed = (BORROW_RATIO * collateral).round() as u128; // health factor 1.25
        self.increase_balance(&account_id, &token, borrowed);
        let mut reserve = self.reserves.get(&token).expect("Reserve not found");
        let pool = &self.pools[pool_id];
        let token = &pool.token1;
        reserve.borrowed += borrowed;
        assert!(
            reserve.deposited >= reserve.borrowed,
            "You want to borrow {} of {} but only {} is available in reserve",
            reserve.borrowed,
            token,
            reserve.deposited
        );
        self.reserves.insert(&token, &reserve);
        let borrow = Borrow {
            owner_id: account_id,
            asset: token.to_string(),
            borrowed,
            collateral: collateral as u128,
            position_id,
            pool_id,
            last_update_timestamp: env::block_timestamp(),
            apr: APR_BORROW,
            leverage: None,
            fees: 0,
        };
        self.borrows.insert(&self.borrows_number, &borrow);
        self.borrows_number += 1;
        self.nft_transfer(
            env::current_account_id(),
            position_id.to_string(),
            None,
            None,
        );
        return borrowed.into();
    }

    pub fn supply_collateral_and_borrow_leveraged(
        &mut self,
        pool_id: usize,
        position_id: u128,
        leverage: u128,
    ) {
        assert!(leverage > 1);
        let account_id = env::predecessor_account_id();
        let pool = &mut self.pools[pool_id];
        let token0 = pool.token0.clone();
        let token1 = pool.token1.clone();
        let position = pool.positions.get(&position_id).expect("Not found").clone();
        let borrowed0 = position.token0_locked as u128 * (leverage - 1);
        let borrowed1 = position.token1_locked as u128 * (leverage - 1);

        let mut reserve = self.reserves.get(&token0).unwrap();
        reserve.borrowed += borrowed0;
        assert!(reserve.deposited >= reserve.borrowed);
        self.reserves.insert(&token0, &reserve);

        let mut reserve = self.reserves.get(&token1).unwrap();
        reserve.borrowed += borrowed1;
        assert!(reserve.deposited >= reserve.borrowed);
        self.reserves.insert(&token1, &reserve);

        let mut position = pool
            .positions
            .get(&position_id)
            .expect("Position not found")
            .clone();
        let total_locked = position.total_locked as u128;
        position.add_liquidity(
            Some(U128::from(position.token0_locked as u128 * (leverage - 1))),
            None,
            pool.sqrt_price,
        );
        pool.positions.insert(position_id, position);

        let borrow = Borrow {
            owner_id: account_id,
            asset: token1,
            borrowed: total_locked * (leverage - 1),
            collateral: total_locked * leverage,
            position_id,
            pool_id,
            last_update_timestamp: env::block_timestamp(),
            apr: APR_BORROW,
            leverage: Some(leverage),
            fees: 0,
        };
        self.borrows.insert(&self.borrows_number, &borrow);
        self.borrows_number += 1;
        self.nft_transfer(
            env::current_account_id(),
            position_id.to_string(),
            None,
            None,
        );
    }

    pub fn return_collateral_and_repay(&mut self, borrow_id: u128) {
        let account_id = env::predecessor_account_id();
        let borrow = self.borrows.remove(&borrow_id).unwrap();
        let pool = &self.pools[borrow.pool_id];
        let position = pool.positions.get(&borrow.position_id).unwrap();
        let health_factor = position.total_locked / borrow.borrowed as f64;
        assert!(health_factor >= 1.0);
        assert_eq!(account_id, borrow.owner_id);
        if let Some(leverage) = borrow.leverage {
            let mut reserve = self.reserves.get(&borrow.asset).unwrap();
            reserve.borrowed -= borrow.borrowed;
            self.reserves.insert(&borrow.asset, &reserve);
            let pool = &self.pools[borrow.pool_id];
            let position = pool.positions.get(&borrow.position_id).unwrap();
            self.remove_liquidity(
                borrow.pool_id,
                borrow.position_id,
                None,
                Some(U128::from(
                    position.token1_locked as u128 * (leverage - 1) + borrow.fees,
                )),
            );
        } else {
            self.decrease_balance(&account_id, &borrow.asset, borrow.borrowed + borrow.fees);
            let mut reserve = self.reserves.get(&borrow.asset).unwrap();
            reserve.borrowed -= borrow.borrowed;
            self.reserves.insert(&borrow.asset, &reserve);
        }
        println!("receiver = {account_id}");
        ext_self::nft_transfer(
            account_id,
            borrow.position_id.to_string(),
            None,
            None,
            &env::current_account_id(),
            0,
            100000000000000,
        );
    }

    pub fn get_liquidation_list(&self) -> Vec<BorrowId> {
        let mut result = Vec::new();
        for (id, _) in self.borrows.iter() {
            if self.get_borrow_health_factor(id) < 1.0 {
                result.push(id);
            }
        }
        result
    }

    pub fn get_borrow_health_factor(&self, borrow_id: BorrowId) -> f64 {
        if let Some(borrow) = self.borrows.get(&borrow_id) {
            let pool = &self.pools[borrow.pool_id];
            let position = pool.positions.get(&borrow.position_id).unwrap();
            return position.total_locked / borrow.borrowed as f64;
        } else {
            return 0.0;
        }
    }

    pub fn liquidate(&mut self, borrow_id: BorrowId) {
        let account_id = env::predecessor_account_id();
        if let Some(borrow) = self.borrows.remove(&borrow_id) {
            let pool = &self.pools[borrow.pool_id];
            let position = pool.positions.get(&borrow.position_id).unwrap();
            let health_factor = position.total_locked / borrow.borrowed as f64;
            assert!(health_factor < 1.0);
            let discount = (1.0 + health_factor) / 2.0;
            let amount = borrow.collateral as f64;
            let discounted_collateral_sum =
                (amount * discount / borrow.leverage.unwrap_or(1) as f64) as u128;
            self.decrease_balance(&account_id, &borrow.asset, discounted_collateral_sum);
            self.nft_transfer(account_id, borrow.position_id.to_string(), None, None);
        } else {
            panic!("Borrow not found");
        }
    }
}
