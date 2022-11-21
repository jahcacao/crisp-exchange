use std::collections::HashMap;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{collections::UnorderedMap, AccountId};

use crate::errors::{
    NOT_ENOUGH_TOKENS, TOKEN_HAS_NOT_BEEN_DEPOSITED, YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL,
};

pub const GAS_FOR_FT_TRANSFER: u64 = 20_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountsInfo {
    pub accounts_info: UnorderedMap<AccountId, Balance>,
}

type Balance = UnorderedMap<AccountId, u128>;

impl AccountsInfo {
    pub fn get_balance(&self, account_id: &AccountId) -> Option<Balance> {
        self.accounts_info.get(account_id)
    }

    pub fn deposit_ft(&mut self, account_id: &AccountId, token_in: &AccountId, amount: u128) {
        if let Some(mut balance) = self.get_balance(account_id) {
            let current_value = balance.get(token_in).unwrap_or(0);
            let new_value = current_value + amount;
            balance.insert(token_in, &new_value);
            self.accounts_info.insert(account_id, &balance);
        } else {
            let mut balance = UnorderedMap::new(account_id.clone().into_bytes());
            balance.insert(&token_in.to_string(), &amount);
            self.accounts_info.insert(account_id, &balance);
        }
    }

    pub fn withdraw(&mut self, account_id: &AccountId, token: &AccountId, amount: u128) {
        if let Some(mut balance) = self.get_balance(account_id) {
            if let Some(current_amount) = balance.get(token) {
                let message = format!(
                    "Not enough tokens. You want to withdraw {} of {} but only have {}",
                    amount, token, current_amount
                );
                assert!(amount <= current_amount, "{}", message);
                balance.insert(token, &(current_amount - amount));
                self.accounts_info.insert(account_id, &balance);
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
        if let Some(mut balance) = self.get_balance(account_id) {
            if let Some(current_amount) = balance.get(token) {
                let message = format!("Not enough tokens. You want to decrease your balance on {} of {} but only have {}", amount, token, current_amount);
                assert!(amount <= current_amount, "{}", message);
                balance.insert(token, &(current_amount - amount));
                self.accounts_info.insert(account_id, &balance);
            }
        } else {
            let message = format!(
                "Not enough tokens. You want to decrease your balance on {} of {} but only have {}",
                amount, token, 0
            );
            panic!("{}", message);
        }
    }

    pub fn increase_balance(&mut self, account_id: &AccountId, token: &AccountId, amount: u128) {
        if let Some(mut balance) = self.get_balance(account_id) {
            if let Some(current_amount) = balance.get(token) {
                balance.insert(token, &(current_amount + amount));
                self.accounts_info.insert(account_id, &balance);
            } else {
                balance.insert(token, &amount);
                self.accounts_info.insert(account_id, &balance);
            }
        } else {
            panic!("{}", YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL);
        }
    }

    pub fn apply_collected_fees(
        &mut self,
        collected_fees: &HashMap<AccountId, f64>,
        token: &AccountId,
    ) {
        for (account_id, amount) in collected_fees.iter() {
            self.increase_balance(account_id, token, *amount as u128);
        }
    }
}
