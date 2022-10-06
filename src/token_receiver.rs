use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{env, json_types::U128, near_bindgen, AccountId, PromiseOrValue};

use crate::*;

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is either "" for deposit or `TokenReceiverMessage`.
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        let message = format!(
            "ft on transfer sender_id = {} token_in = {} msg = {}",
            sender_id, token_in, msg
        );
        env::log_str(message.as_str());
        self.accounts
            .deposit_ft(&sender_id, &token_in, amount.into());
        PromiseOrValue::Value(U128(0))
    }
}
