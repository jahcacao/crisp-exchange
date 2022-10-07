use near_sdk::json_types::U128;
use near_sdk::test_utils::accounts;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;

use crate::common::utils::deposit_tokens;
use crate::common::utils::setup_contract;

mod common;

#[test]
fn create_pool() {
    let (mut _context, mut contract) = setup_contract();
    contract.create_pool(accounts(0).to_string(), accounts(1).to_string());
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.tokens[0] == accounts(0).to_string());
    assert!(pool.tokens[1] == accounts(1).to_string());
    assert!(pool.liquidity[0] == 0);
    assert!(pool.liquidity[1] == 0);
}

#[test]
fn add_liquidity() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string());
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 100000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.add_liquidity(0, accounts(1).to_string(), 1000);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity[0] == 1000);
    assert!(pool.liquidity[1] == 0);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 99000);
    let share = pool.get_share(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(share, 1000);
}

#[test]
#[should_panic(expected = "You have not added liquidity to this pool")]
fn test_remove_liquidity_without_depositing() {
    let (mut _context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string());
    contract.remove_liquidity(0, accounts(1).to_string(), 1);
}

#[test]
#[should_panic(expected = "You want to remove too much liquidity")]
fn test_remove_liquidity_too_many_tokens() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string());
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 100000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.add_liquidity(0, accounts(1).to_string(), 1000);
    contract.remove_liquidity(0, accounts(1).to_string(), 2000);
}

#[test]
fn withdraw_liquidity() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string());
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 100000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.add_liquidity(0, accounts(1).to_string(), 1000);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity[0] == 1000);
    assert!(pool.liquidity[1] == 0);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 99000);
    let share = pool.get_share(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(share, 1000);

    contract.remove_liquidity(0, accounts(1).to_string(), 1000);

    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity[0] == 0);
    assert!(pool.liquidity[1] == 0);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 100000);
    let share = pool.get_share(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(share, 0);
}
