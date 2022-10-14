use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{collections::UnorderedMap, AccountId};

use crate::errors::{
    NOT_ENOUGH_TOKENS, TOKEN_HAS_NOT_BEEN_DEPOSITED, YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL,
};
use crate::StorageKey;

pub const GAS_FOR_FT_TRANSFER: u64 = 20_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountsInfo {
    pub accounts_info: UnorderedMap<AccountId, Balance>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Balance {
    pub balance: UnorderedMap<AccountId, u128>,
}

impl Balance {
    pub fn new(account_id: AccountId) -> Balance {
        Balance {
            balance: UnorderedMap::new(account_id.into_bytes()),
        }
    }
}

impl AccountsInfo {
    pub fn new() -> AccountsInfo {
        AccountsInfo {
            accounts_info: UnorderedMap::new(StorageKey::Accounts),
        }
    }

    pub fn get_balance(&self, account_id: &AccountId) -> Option<Balance> {
        self.accounts_info.get(account_id)
    }

    pub fn deposit_ft(&mut self, account_id: &AccountId, token_in: &AccountId, amount: u128) {
        if let Some(mut balance) = self.get_balance(&account_id.to_string()) {
            let current_value = balance.balance.get(token_in).unwrap_or(0);
            let new_value = current_value + amount;
            balance.balance.insert(&token_in.to_string(), &new_value);
            self.accounts_info.insert(&account_id.to_string(), &balance);
        } else {
            let mut balance = Balance::new(account_id.clone());
            balance.balance.insert(&token_in.to_string(), &amount);
            self.accounts_info.insert(&account_id.to_string(), &balance);
        }
    }

    pub fn withdraw(&mut self, account_id: AccountId, token: AccountId, amount: u128) {
        if let Some(mut balance) = self.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token.to_string()) {
                assert!(amount <= current_amount, "{}", NOT_ENOUGH_TOKENS);
                balance.balance.insert(&token, &(current_amount - amount));
                self.accounts_info.insert(&account_id, &balance);
                ext_fungible_token::ft_transfer(
                    account_id.to_string(),
                    U128(amount),
                    None,
                    &token,
                    1,
                    GAS_FOR_FT_TRANSFER,
                );
                return;
            }
        }
        panic!("{}", TOKEN_HAS_NOT_BEEN_DEPOSITED);
    }

    pub fn decrease_balance(&mut self, account_id: &AccountId, token: &AccountId, amount: u128) {
        if let Some(mut balance) = self.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token) {
                assert!(amount <= current_amount, "{}", NOT_ENOUGH_TOKENS);
                balance.balance.insert(&token, &(current_amount - amount));
                self.accounts_info.insert(&account_id, &balance);
            }
        } else {
            panic!("{}", NOT_ENOUGH_TOKENS);
        }
    }

    pub fn increase_balance(&mut self, account_id: &AccountId, token: &AccountId, amount: u128) {
        if let Some(mut balance) = self.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token) {
                balance.balance.insert(&token, &(current_amount + amount));
                self.accounts_info.insert(&account_id, &balance);
            }
        } else {
            panic!("{}", YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL);
        }
    }
}
