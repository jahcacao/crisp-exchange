use crate::common::utils::{deposit_tokens, setup_contract, withdraw_tokens};
use near_sdk::MockedBlockchain;
use near_sdk::{json_types::U128, test_utils::accounts, testing_env};

mod common;

#[test]
fn test_balance_after_deposit() {
    let (mut context, mut contract) = setup_contract();
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(123456),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(123456));
}

#[test]
fn test_balance_after_two_deposits() {
    let (mut context, mut contract) = setup_contract();
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(10000),
    );
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(30000));
}

#[test]
fn test_balance_after_withdraw() {
    let (mut context, mut contract) = setup_contract();
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    testing_env!(context.signer_account_id(accounts(0)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(10000),
    );
    withdraw_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(10000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(0));
}

#[test]
#[should_panic(expected = "Not enough tokens")]
fn test_balance_withdraw_not_enough_token() {
    let (mut context, mut contract) = setup_contract();
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    testing_env!(context.signer_account_id(accounts(0)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(10000),
    );
    withdraw_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
}

#[test]
#[should_panic(expected = "Token has not been deposited")]
fn test_balance_withdraw_without_deposit() {
    let (mut context, mut contract) = setup_contract();
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    withdraw_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
}

#[test]
fn test_balance_after_two_deposits_two_accounts() {
    let (mut context, mut contract) = setup_contract();
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(10000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(20000),
    );
    testing_env!(context.predecessor_account_id(accounts(3)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(1),
        U128(30000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(2),
        U128(40000),
    );
    let balance1 = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2 = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    let balance3 = contract.get_balance(&accounts(3).to_string(), &accounts(1).to_string());
    let balance4 = contract.get_balance(&accounts(3).to_string(), &accounts(2).to_string());
    assert_eq!(balance1, U128(10000));
    assert_eq!(balance2, U128(20000));
    assert_eq!(balance3, U128(30000));
    assert_eq!(balance4, U128(40000));
}
