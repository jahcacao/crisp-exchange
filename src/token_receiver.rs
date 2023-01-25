use crate::Contract;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::ValidAccountId;
use near_sdk::serde_json;
use near_sdk::{env, json_types::U128, near_bindgen, PromiseOrValue};

use crate::action::Action;
use crate::*;

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    Execute { actions: Vec<Action> },
}

impl Contract {
    fn internal_execute(&mut self, token_in: AccountId, actions: &[Action]) {
        for action in actions {
            match action {
                Action::Swap(swap_action) => {
                    assert_eq!(token_in, swap_action.token_in);
                    self.swap(
                        swap_action.pool_id,
                        &swap_action.token_in,
                        swap_action.amount_in,
                        &swap_action.token_out,
                    );
                }
                Action::Withdraw(withdraw_action) => {
                    self.withdraw(&withdraw_action.token, withdraw_action.amount);
                }
                Action::MultihopeSwap(_) => todo!(),
                Action::OpenPosition(_) => todo!(),
                Action::AddLiquidity(_) => todo!(),
                Action::CreateDeposit(_) => todo!(),
                Action::ReturnCollateralAndRepay(_) => todo!(),
                Action::Liquidate(_) => todo!(),
            }
        }
    }
}

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
        if msg == "".to_string() {
            return PromiseOrValue::Value(U128(0));
        }
        // instant swap
        let message = serde_json::from_str::<TokenReceiverMessage>(&msg).expect("Wrong msg format");
        match message {
            TokenReceiverMessage::Execute { actions } => {
                self.internal_execute(token_in, &actions);
                return PromiseOrValue::Value(U128(0));
            }
        }
        PromiseOrValue::Value(U128(0))
    }
}

#[cfg(test)]
mod test {
    use crate::action::{SwapAction, WithdrawAction};

    use super::*;

    #[test]
    fn message_test() {
        let swap_action = Action::Swap(SwapAction {
            pool_id: 0,
            token_in: "token_in.testnet".to_string(),
            amount_in: U128(1000),
            token_out: "token_out.testnet".to_string(),
        });
        let withdraw_action = Action::Withdraw(WithdrawAction {
            token: "token_in.testnet".to_string(),
            amount: U128(1000),
        });
        let token_receiver_message = TokenReceiverMessage::Execute {
            actions: vec![swap_action, withdraw_action],
        };
        let message: String = serde_json::to_value(&token_receiver_message)
            .unwrap()
            .to_string();
        println!("message is: {}", message);
    }
}
