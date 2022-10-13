use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{collections::UnorderedMap, AccountId};

use crate::StorageKey;

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
}
