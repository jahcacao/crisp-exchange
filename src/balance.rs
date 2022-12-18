use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::json_types::U128;
use near_sdk::{collections::UnorderedMap, AccountId};
use std::collections::HashMap;

use crate::errors::{TOKEN_HAS_NOT_BEEN_DEPOSITED, YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL};
use crate::pool::CollectedFee;
use crate::Contract;

pub const GAS_FOR_FT_TRANSFER: u64 = 20_000_000_000_000;

pub type BalancesMap = UnorderedMap<AccountId, Balance>;
type Balance = UnorderedMap<AccountId, u128>;

impl Contract {
    pub fn deposit_ft(&mut self, account_id: &AccountId, token_in: &AccountId, amount: u128) {
        if let Some(mut balance) = self.balances_map.get(account_id) {
            let current_value = balance.get(token_in).unwrap_or(0);
            let new_value = current_value + amount;
            balance.insert(token_in, &new_value);
            self.balances_map.insert(account_id, &balance);
        } else {
            let mut balance = UnorderedMap::new(account_id.clone().into_bytes());
            balance.insert(&token_in.to_string(), &amount);
            self.balances_map.insert(account_id, &balance);
        }
    }

    pub fn balance_withdraw(&mut self, account_id: &AccountId, token: &AccountId, amount: u128) {
        if let Some(mut balance) = self.balances_map.get(account_id) {
            if let Some(current_amount) = balance.get(token) {
                let message = format!(
                    "Not enough tokens. You want to withdraw {} of {} but only have {}",
                    amount, token, current_amount
                );
                assert!(amount <= current_amount, "{}", message);
                balance.insert(token, &(current_amount - amount));
                self.balances_map.insert(account_id, &balance);
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
        if let Some(mut balance) = self.balances_map.get(account_id) {
            if let Some(current_amount) = balance.get(token) {
                let message = format!("Not enough tokens. You want to decrease your balance on {} of {} but only have {}", amount, token, current_amount);
                assert!(amount <= current_amount, "{}", message);
                balance.insert(token, &(current_amount - amount));
                self.balances_map.insert(account_id, &balance);
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
        if let Some(mut balance) = self.balances_map.get(account_id) {
            if let Some(current_amount) = balance.get(token) {
                balance.insert(token, &(current_amount + amount));
                self.balances_map.insert(account_id, &balance);
            } else {
                balance.insert(token, &amount);
                self.balances_map.insert(account_id, &balance);
            }
        } else {
            panic!("{}", YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL);
        }
    }

    pub fn apply_collected_fees(
        &mut self,
        collected_fees: &HashMap<u128, CollectedFee>,
        token: &AccountId,
    ) {
        for (_, collected_fee) in collected_fees {
            self.increase_balance(
                &collected_fee.account_id,
                token,
                collected_fee.amount.round() as u128,
            );
        }
    }
}
