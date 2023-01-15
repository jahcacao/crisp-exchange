use std::collections::HashMap;

use near_sdk::env;
use near_sdk::json_types::ValidAccountId;
use near_sdk::json_types::U128;
use near_sdk::serde_json;
use near_sdk::test_utils::accounts;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;

use crate::common::utils::deposit_tokens;
use crate::common::utils::setup_contract;

mod common;

#[test]
fn create_pool() {
    let (mut _context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(0).to_string(),
        accounts(1).to_string(),
        100.0,
        0,
        0,
    );
    let pool = contract.get_pool(0);
    assert!(pool.token0 == accounts(0).to_string());
    assert!(pool.token1 == accounts(1).to_string());
    assert!(pool.liquidity == 0.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions == HashMap::new());
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.protocol_fee == 0);
    assert!(pool.rewards == 0);
}

#[test]
fn open_position_is_correct() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(50),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(50));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(27505),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(27505));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    let pool = contract.get_pool(0);
    println!("pool.liquidity = {}", pool.liquidity);
    assert!(pool.liquidity == 5500.834197154125);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(0));
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(0));
}

#[test]
fn open_position_less_than_lower_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(2000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(3000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(50)), None, 121.0, 144.0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(1950));
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(3000));
}

#[test]
fn open_position_more_than_upper_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(2000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(3000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(U128(50)), 64.0, 81.0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(2000));
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(2950));
}

#[test]
fn open_two_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(20000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(30000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(U128(50)), 64.0, 121.0);
    contract.open_position(0, Some(U128(100)), None, 49.0, 144.0);
    let pool = contract.get_pool(0);
    println!("pool.liquidity = {}", pool.liquidity);
    assert!(pool.liquidity == 6025.922352607511);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 2);
}

#[test]
fn open_three_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(20000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(30000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(U128(50)), 64.0, 121.0);
    contract.open_position(0, Some(U128(100)), None, 49.0, 144.0);
    contract.open_position(0, None, Some(U128(150)), 81.0, 169.0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity.round() == 6176.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 3);
}

#[test]
fn open_ten_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(2000000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(3000000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(U128(50)), 64.0, 121.0);
    contract.open_position(0, Some(U128(100)), None, 49.0, 144.0);
    contract.open_position(0, None, Some(U128(150)), 81.0, 169.0);
    contract.open_position(0, Some(U128(200)), None, 110.0, 121.0);
    contract.open_position(0, None, Some(U128(250)), 49.0, 99.0);
    contract.open_position(0, Some(U128(300)), None, 149.0, 154.0);
    contract.open_position(0, None, Some(U128(350)), 81.0, 99.0);
    contract.open_position(0, Some(U128(100)), None, 49.0, 144.0);
    contract.open_position(0, None, Some(U128(50)), 64.0, 121.0);
    contract.open_position(0, Some(U128(500)), None, 120.0, 130.0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity.round() == 12202.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 10);
}

#[test]
fn close_position() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(20000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(30000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(U128(50)), 64.0, 121.0);
    contract.close_position(0, 0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 0);
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(20000));
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(30000));
}

#[test]
fn close_two_position() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(2000000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(3000000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100)), None, 49.0, 144.0);
    contract.open_position(0, Some(U128(100)), None, 49.0, 144.0);
    contract.close_position(0, 1);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity == 6000.926902650581);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    contract.close_position(0, 0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 0);
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(2000000));
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(3000000));
}

#[test]
fn get_expense() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(10000000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(10000000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(1100507792),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(1100507792));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(10000000)), None, 81.0, 121.0);
    let result1 = contract.get_expense(0, &accounts(1).to_string(), U128(1));
    let result2 = contract.get_expense(0, &accounts(2).to_string(), U128(1000));
    let result3 = contract.get_expense(0, &accounts(1).to_string(), U128(10005000));
    let result4 = contract.get_expense(0, &accounts(2).to_string(), U128(1101002812));
    let pool = &contract.pools[0];
    let position = &pool.positions.get(&0).unwrap();
    println!("result1 = {}", result1.0);
    println!("result2 = {}", result2.0);
    println!("result3 = {}", result3.0);
    println!("result4 = {}", result4.0);
    println!("token0 locked = {}", pool.token0_locked);
    println!("token1 locked = {}", pool.token1_locked);
    println!("liquidity = {}", position.liquidity);
    println!("pool liquidity = {}", pool.liquidity);
    assert!(result1 == U128(100));
    assert!(result2 == U128(10));
}

#[test]
fn swap_in_token0() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(200000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(11005078),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(100000));
    assert!(balance2_before == U128(0));
    let amount1 = 100000;
    let amount2 = contract.swap(
        0,
        accounts(1).to_string(),
        U128(amount1),
        accounts(2).to_string(),
    );
    let balance1_after = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_after == U128(0));
    assert!(balance2_after == amount2);
}

#[test]
fn swap_in_token1() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(11105078),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(0));
    println!("balance2_before = {}", balance2_before.0);
    assert!(balance2_before == U128(100000));
    let amount1 = 100000;
    let amount2 = contract.swap(
        0,
        accounts(2).to_string(),
        U128(amount1),
        accounts(1).to_string(),
    );
    let balance1_after = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_after == amount2);
    assert!(balance2_after == U128(0));
}

#[test]
fn fee_test() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        100,
        100,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(11005078),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(1),
        U128(0),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(2),
        U128(100000),
    );
    let balance1_before = contract.get_balance(&accounts(3).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(3).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(0));
    assert!(balance2_before == U128(100000));
    let amount1 = 100000;
    testing_env!(context.predecessor_account_id(accounts(3)).build());
    let result: u128 = contract
        .swap(
            0,
            accounts(2).to_string(),
            U128(amount1),
            accounts(1).to_string(),
        )
        .into();
    let balance1_after: u128 = contract
        .get_balance(&accounts(3).to_string(), &accounts(1).to_string())
        .into();
    let balance2_after: u128 = contract
        .get_balance(&accounts(3).to_string(), &accounts(2).to_string())
        .into();
    let amount2 = result as f64 * 0.98;
    assert!((balance1_after as f64 - amount2).abs() < 10.0);
    assert!(balance2_after == 0);
    let balance1_lp_after: u128 = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .into();
    let balance2_lp_after: u128 = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .into();
    let amount3 = result as f64 * 0.01;
    assert!((balance1_lp_after as f64 - amount3).abs() < 10.0);
    assert!(balance2_lp_after == 0);
}

#[test]
fn collected_fee() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        100,
        100,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(11000000),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(50000)), None, 81.0, 121.0);
    contract.open_position(0, Some(U128(50000)), None, 91.0, 111.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(1),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(2),
        U128(100000),
    );
    let balance1_before = contract.get_balance(&accounts(3).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(3).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(100000));
    assert!(balance2_before == U128(100000));
    let amount1 = 100000;
    testing_env!(context.predecessor_account_id(accounts(3)).build());
    let _pool = &contract.pools[0];
    let _result: u128 = contract
        .swap(
            0,
            accounts(2).to_string(),
            U128(amount1),
            accounts(1).to_string(),
        )
        .into();
    let _pool = &contract.pools[0];
    let _result: u128 = contract
        .swap(
            0,
            accounts(1).to_string(),
            U128(99001),
            accounts(2).to_string(),
        )
        .into();
    let pool = &contract.pools[0];
    let position = pool.positions.get(&0).unwrap();
    assert!(position.fees_earned_token0 == 4);
    println!(
        "pool.positions[0].fees_earned_token1 = {}",
        position.fees_earned_token1
    );
    assert!(position.fees_earned_token1 == 46522);
    println!(
        "pool.positions[0].fees_earned_token1 = {}",
        position.fees_earned_token1
    );
    let position = pool.positions.get(&1).unwrap();
    assert!(position.fees_earned_token0 == 6);
    println!(
        "pool.positions[1].fees_earned_token1 = {}",
        position.fees_earned_token1
    );
    assert!(position.fees_earned_token1 == 46007);
}

#[test]
fn value_locked_open_close() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        100,
        100,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    let initial_balance1 = 100000;
    let initial_balance2 = 11005077;
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(11005078),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let pool = &contract.pools[0];
    assert!(pool.token0_locked == 100000);
    assert!(pool.token1_locked == 11005078);
    contract.close_position(0, 0);
    let pool = &contract.pools[0];
    assert!(pool.token0_locked == 0);
    assert!(pool.token1_locked == 0);

    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let pool = &contract.pools[0];
    assert!(pool.token0_locked == 100000);
    assert!(pool.token1_locked == 11005078);

    contract.close_position(0, 1);
    let pool = &contract.pools[0];
    assert!(pool.token0_locked == 0);
    assert!(pool.token1_locked == 0);
    let final_balance1 = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let final_balance2 = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(initial_balance1 == final_balance1.0);
    assert!(((initial_balance2 as f64).abs() - (final_balance2.0 as f64).abs()) <= 1.0);
}

#[test]
fn value_locked_swap() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    let initial_balance1 = 200000;
    let initial_balance2 = 11005078;
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(initial_balance1),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    contract.swap(
        0,
        accounts(1).to_string(),
        U128(100000),
        accounts(2).to_string(),
    );
    contract.close_position(0, 0);
    let balance1 = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2 = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1.0 == 200000);
    assert!(balance2.0 == 11005078);
}

#[test]
fn value_locked_more_open() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        100,
        100,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    let initial_balance1 = 100000;
    let initial_balance2 = 11005100;
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(initial_balance1),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    for _ in 0..100 {
        contract.open_position(0, Some(U128(1000)), None, 81.0, 121.0);
        let pool = &contract.pools[0];
        assert!(pool.token0_locked <= initial_balance1);
        assert!(pool.token1_locked <= initial_balance2);
    }
    let pool = &contract.pools[0];
    assert!(pool.token0_locked == 100000);
    assert!(pool.token1_locked == 11005078);
}

#[test]
fn value_locked_more_swaps() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        10000.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    let initial_balance1 = 101000;
    let initial_balance2 = 10763056;
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(initial_balance1),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 9990.0, 11000.0);
    for _ in 0..10 {
        contract.swap(
            0,
            accounts(1).to_string(),
            U128(100),
            accounts(2).to_string(),
        );
        let pool = &contract.pools[0];
        let position = &pool.positions.get(&0).unwrap();
        assert!(pool.token0_locked == (position.token0_locked.round() as u128));
        assert!(pool.token1_locked == (position.token1_locked.round() as u128));
        assert!(pool.token0_locked <= initial_balance1);
        assert!(pool.token1_locked <= initial_balance2);
        let balance1 = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
        let balance2 = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
        assert!((balance1.0 + pool.token0_locked) <= initial_balance1);
        assert!((balance2.0 + pool.token1_locked) <= (initial_balance2 + 2));
    }
    contract.close_position(0, 0);
    let balance1 = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2 = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1.0 <= initial_balance1);
    assert!(balance2.0 <= (initial_balance2 + 2));
}

#[test]
fn add_and_remove_liquidity1() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        10000.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    let initial_balance1 = 101000;
    let initial_balance2 = 10763056;
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(initial_balance1),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 9990.0, 11000.0);
    contract.remove_liquidity(0, 0, Some(U128(10000)), None);
    contract.add_liquidity(0, 0, Some(U128(10000)), None);
    let pool = &contract.pools[0];
    let position = &pool.positions.get(&0).unwrap();
    assert!(position.token0_locked.round() == 100000.0);
}

#[test]
fn add_and_remove_liquidity2() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        10000.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    let initial_balance1 = 101000;
    let initial_balance2 = 10763056;
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(initial_balance1),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(U128(100000)), 9990.0, 11000.0);
    contract.remove_liquidity(0, 0, None, Some(U128(10000)));
    contract.add_liquidity(0, 0, None, Some(U128(10000)));
    let pool = &contract.pools[0];
    let position = &pool.positions.get(&0).unwrap();
    assert!(position.token1_locked.round() == 100000.0);
}

#[test]
fn open_many_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    for i in 3..103 {
        let account = format!("\"{i}.testnet\"");
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(1),
            U128(2000000),
        );
        testing_env!(context.predecessor_account_id(accounts(2)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(2),
            U128(3000000),
        );
        testing_env!(context
            .predecessor_account_id(serde_json::from_str(account.as_str()).unwrap())
            .build());
        for _ in 0..10 {
            contract.open_position(0, Some(U128(50)), None, 64.0, 121.0);
        }
    }
    let pool = &contract.pools[0];
    println!("len = {}", pool.positions.len());
    assert!(pool.positions.len() == 1000);
}

#[test]
fn open_many_positions_with_swap1() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    for i in 3..13 {
        let account = format!("\"{i}.testnet\"");
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(1),
            U128(2000000),
        );
        testing_env!(context.predecessor_account_id(accounts(2)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(2),
            U128(3000000),
        );
        testing_env!(context
            .predecessor_account_id(serde_json::from_str(account.as_str()).unwrap())
            .build());
        for _ in 0..10 {
            contract.open_position(0, Some(U128(50)), None, 64.0, 121.0);
        }
        let amount = contract.swap(
            0,
            accounts(1).to_string(),
            U128(10),
            accounts(2).to_string(),
        );
        contract.swap(0, accounts(2).to_string(), amount, accounts(1).to_string());
    }
    let pool = &contract.pools[0];
    println!("len = {}", pool.positions.len());
    assert!(pool.positions.len() == 100);
}

#[test]
fn open_many_positions_with_swap2() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    for i in 3..153 {
        let account = format!("\"{i}.testnet\"");
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(1),
            U128(2000000),
        );
        testing_env!(context.predecessor_account_id(accounts(2)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(2),
            U128(3000000),
        );
        testing_env!(context
            .predecessor_account_id(serde_json::from_str(account.as_str()).unwrap())
            .build());
        contract.open_position(0, Some(U128(50)), None, 64.0, 121.0);
        let amount = contract.swap(
            0,
            accounts(1).to_string(),
            U128(10),
            accounts(2).to_string(),
        );
        contract.swap(0, accounts(2).to_string(), amount, accounts(1).to_string());
    }
    let pool = &contract.pools[0];
    println!("len = {}", pool.positions.len());
    assert!(pool.positions.len() == 150);
}

#[test]
fn create_reserve() {
    let (mut _context, mut contract) = setup_contract();
    assert!(contract.reserves.is_empty());
    contract.create_reserve("usdt.testnet".to_string());
    let reserve = contract.reserves.get(&"usdt.testnet".to_string()).unwrap();
    assert_eq!(reserve.deposited, 0);
    assert_eq!(reserve.borrowed, 0);
    assert_eq!(reserve.liquidation_threshold, 0.0);
    assert_eq!(reserve.loan_to_value, 0.0);
    assert_eq!(reserve.target_utilization_rate, 0.0);
    assert_eq!(reserve.total_liquidity, 0);
    assert_eq!(reserve.utilization_rate, 0.0);
}

#[should_panic]
#[test]
fn create_deposit1() {
    let (mut _context, mut contract) = setup_contract();
    contract.create_reserve("usdt.testnet".to_string());
    contract.create_deposit("usn.testnet".to_string(), U128::from(100));
}

#[test]
fn create_deposit2() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(accounts(1).into(), U128::from(100));
    let deposit = contract.deposits.get(&0).unwrap();
    assert_eq!(deposit.owner_id, accounts(0).to_string());
    assert_eq!(deposit.asset, accounts(1).to_string());
    assert_eq!(deposit.amount, 100);
    assert_eq!(deposit.timestamp, 0);
    assert_eq!(deposit.last_update_timestamp, 0);
    assert_eq!(deposit.apr, 500);
    assert_eq!(deposit.growth, 0);
}

#[test]
fn close_deposit() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(accounts(1).into(), U128::from(100));
    contract.deposits.get(&0).unwrap();
    contract.close_deposit(U128::from(0));
    assert!(contract.deposits.is_empty());
}

#[test]
fn refresh_deposit_growth() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(300),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(accounts(1).into(), U128::from(100));
    contract.create_deposit(accounts(1).into(), U128::from(200));
    let context = context.block_timestamp(31536000000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    context.block_index(31536000);
    context.epoch_height(31536000);
    assert_eq!(context.context.block_timestamp, 31536000000);
    assert_eq!(context.context.block_index, 31536000);
    assert_eq!(context.context.epoch_height, 31536000);
    contract.refresh_deposits_growth();
    let deposit1 = contract.deposits.get(&0).unwrap();
    let deposit2 = contract.deposits.get(&1).unwrap();
    assert_eq!(deposit1.growth, 5);
    assert_eq!(deposit2.growth, 10);
}

#[test]
fn take_deposit_growth() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(accounts(1).into(), U128::from(100));
    let context = context.block_timestamp(31536000000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    context.block_index(31536000);
    context.epoch_height(31536000);
    contract.refresh_deposits_growth();
    let deposit = contract.deposits.get(&0).unwrap();
    assert_eq!(deposit.growth, 5);
    let growth = contract.take_deposit_growth(U128::from(0), U128::from(10));
    assert_eq!(growth, 5.into());
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128::from(5));
}

#[test]
fn get_account_deposits() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(accounts(1).into(), U128::from(10));
    contract.create_deposit(accounts(1).into(), U128::from(20));
    contract.create_deposit(accounts(1).into(), U128::from(30));
    contract.create_deposit(accounts(1).into(), U128::from(40));
    let deposits = contract.get_account_deposits(accounts(0).to_string());
    println!("deposits = {:#?}", deposits);
}

#[should_panic(expected = "Reserve not found")]
#[test]
fn supply_collateral_and_borrow_simple_panic() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(50),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(27505),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.supply_collateral_and_borrow_simple(0, 0);
}

#[should_panic(
    expected = "You want to borrow 26004 of charlie but only 10 is available in reserve"
)]
#[test]
fn supply_collateral_and_borrow_simple_not_enough_reserves() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(2).into());
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(50),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(27515),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.create_deposit(accounts(2).into(), U128::from(10));
    contract.supply_collateral_and_borrow_simple(0, 0);
}

#[test]
fn supply_collateral_and_borrow_simple_should_work() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(2).into());
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(50),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(127515),
    );
    testing_env!(context
        .predecessor_account_id(accounts(0))
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.create_deposit(accounts(2).into(), U128::from(100000));
    let balance_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    let borrowed = contract.supply_collateral_and_borrow_simple(0, 0);
    let balance_after = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance_before.0 + borrowed.0, balance_after.0);
    let borrow = contract.borrows.get(&0).unwrap();
    let pool = &contract.pools[0];
    let position = pool.positions.get(&0).unwrap();
    assert_eq!(borrow.owner_id, accounts(0).to_string());
    assert_eq!(borrow.asset, accounts(2).to_string());
    assert_eq!(borrow.borrowed, borrowed.0);
    assert_eq!(borrow.collateral, position.total_locked as u128);
    assert_eq!(borrow.position_id, 0);
    assert_eq!(borrow.pool_id, 0);
    assert_eq!(borrow.health_factor, 1.25);
    assert_eq!(borrow.last_update_timestamp, 0);
    assert_eq!(borrow.apr, 1000);
    assert_eq!(borrow.leverage, None);
    assert_eq!(borrow.fees, 0);
    let token = contract.tokens_by_id.get(&"0".to_string()).unwrap();
    assert_eq!(token.owner_id, context.context.current_account_id);
}

#[test]
fn supply_collateral_and_borrow_leveraged() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(1).into());
    contract.create_reserve(accounts(2).into());
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100050),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(127515),
    );
    testing_env!(context
        .predecessor_account_id(accounts(0))
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    let pool = &contract.pools[0];
    let position = pool.positions.get(&0).unwrap();
    let total_locked = position.total_locked as u128;
    contract.create_deposit(accounts(1).into(), U128::from(100000));
    contract.create_deposit(accounts(2).into(), U128::from(100000));
    let balance_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    let leverage = 2;
    contract.supply_collateral_and_borrow_leveraged(0, 0, leverage);
    let balance_after = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance_before.0, balance_after.0);
    let borrow = contract.borrows.get(&0).unwrap();
    assert_eq!(borrow.owner_id, accounts(0).to_string());
    assert_eq!(borrow.asset, accounts(2).to_string());
    assert_eq!(borrow.borrowed, (leverage - 1) * total_locked);
    assert_eq!(borrow.collateral, leverage * total_locked);
    assert_eq!(borrow.position_id, 0);
    assert_eq!(borrow.pool_id, 0);
    assert_eq!(
        borrow.health_factor,
        leverage as f64 / (leverage as f64 - 1.0)
    );
    assert_eq!(borrow.last_update_timestamp, 0);
    assert_eq!(borrow.apr, 1000);
    assert_eq!(borrow.leverage, Some(leverage));
    assert_eq!(borrow.fees, 0);
    let token = contract.tokens_by_id.get(&"0".to_string()).unwrap();
    assert_eq!(token.owner_id, context.context.current_account_id);
}

#[test]
fn return_collateral_and_repay() {
    let alice = ValidAccountId::try_from("john.near").unwrap();
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(accounts(2).into());
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        alice.clone(),
        accounts(1),
        U128(50),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        alice.clone(),
        accounts(2),
        U128(127515),
    );
    testing_env!(context
        .predecessor_account_id(alice.clone())
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.create_deposit(accounts(2).into(), U128::from(100000));
    let borrowed = contract.supply_collateral_and_borrow_simple(0, 0);
    println!("borrowed = {} of {}", borrowed.0, accounts(2));
    testing_env!(context.predecessor_account_id(alice.clone()).build());
    println!("thieteen");
    let token = contract.tokens_by_id.get(&"0".to_string()).unwrap();
    println!(
        "token.owner_id = {}, context.context.current_account_id = {}",
        token.owner_id, context.context.current_account_id
    );
    let balance1_before = contract.get_balance(&"john.near".to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&"john.near".to_string(), &accounts(2).to_string());
    contract.return_collateral_and_repay(0);
    testing_env!(context.predecessor_account_id(alice).build());
    let balance1_after = contract.get_balance(&"john.near".to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&"john.near".to_string(), &accounts(2).to_string());
    assert_eq!(balance1_before, balance1_after);
    assert_eq!(balance2_before.0 - borrowed.0, balance2_after.0);
}

#[test]
fn get_liquidation_list() {
    let (mut context, mut contract) = setup_contract();
    let list = contract.get_liquidation_list();
    assert!(list.is_empty());
}

#[test]
fn liquidate() {
    let (mut context, mut contract) = setup_contract();
}
