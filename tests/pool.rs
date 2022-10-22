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
    contract.create_pool(accounts(0).to_string(), accounts(1).to_string(), 0.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.token0 == accounts(0).to_string());
    assert!(pool.token1 == accounts(1).to_string());
    assert!(pool.token0_liquidity == 0);
    assert!(pool.token1_liquidity == 0);
    assert!(pool.positions == vec![]);
    assert!(pool.sqrt_price == 0.0);
}

#[test]
fn open_position() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 50.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, 1000, 1000, 25, 100);
    let pool = contract.get_pool(0).unwrap();
    println!("pool.token0_liquidity = {}", pool.token0_liquidity);
    println!("pool.token1_liquidity = {}", pool.token1_liquidity);
    assert!(pool.token0_liquidity == 41);
    assert!(pool.token1_liquidity == 2071);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 1000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
}

#[test]
fn open_position_less_than_lower_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 10.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, 1000, 1000, 25, 36);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.token0_liquidity == 33);
    assert!(pool.token1_liquidity == 0);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 1000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
}

#[test]
fn open_position_less_than_upper_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 40.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, 1000, 1000, 25, 36);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.token0_liquidity == 0);
    assert!(pool.token1_liquidity == 1000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 1000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
}

/*
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

#[test]
fn get_return() {
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
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.add_liquidity(0, accounts(1).to_string(), 1000);
    contract.add_liquidity(0, accounts(2).to_string(), 1000);
    let result = contract.get_return(0 as usize, &accounts(1).to_string(), 200);
    assert!(result == 166);
}

#[test]
fn swap() {
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
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.add_liquidity(0, accounts(1).to_string(), 10000);
    contract.add_liquidity(0, accounts(2).to_string(), 10000);
    contract.swap(0, accounts(1).to_string(), 1000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 89000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 90909);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity[0] == 11000);
    assert!(pool.liquidity[1] == 9091);
}
*/
