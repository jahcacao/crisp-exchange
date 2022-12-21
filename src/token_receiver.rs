use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::ValidAccountId;
use near_sdk::{env, json_types::U128, near_bindgen, PromiseOrValue};

use crate::*;

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        self.deposit_ft(&sender_id.into(), &token_in, amount.into());
        PromiseOrValue::Value(U128(0))
    }
}
