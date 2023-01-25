use std::collections::HashMap;

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
    let _result3 = contract.get_expense(0, &accounts(1).to_string(), U128(10005000));
    let _result4 = contract.get_expense(0, &accounts(2).to_string(), U128(1101002812));
    let pool = &contract.pools[0];
    let _position = &pool.positions.get(&0).unwrap();
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
    testing_env!(context.signer_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(100000));
    assert!(balance2_before == U128(0));
    let amount1 = 100000;
    let amount2 = contract.swap(
        0,
        &accounts(1).to_string(),
        U128(amount1),
        &accounts(2).to_string(),
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
    testing_env!(context.signer_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(0));
    assert!(balance2_before == U128(100000));
    let amount1 = 100000;
    let amount2 = contract.swap(
        0,
        &accounts(2).to_string(),
        U128(amount1),
        &accounts(1).to_string(),
    );
    let balance1_after = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_after == amount2);
    assert!(balance2_after == U128(0));
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
    testing_env!(context.signer_account_id(accounts(1)).build());
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
    testing_env!(context.signer_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    testing_env!(context.signer_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    contract.swap(
        0,
        &accounts(1).to_string(),
        U128(100000),
        &accounts(2).to_string(),
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
    testing_env!(context.signer_account_id(accounts(1)).build());
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
    testing_env!(context.signer_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    testing_env!(context.signer_account_id(accounts(0)).build());
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
    testing_env!(context.signer_account_id(accounts(1)).build());
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
    testing_env!(context.signer_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(initial_balance2),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    testing_env!(context.signer_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 9990.0, 11000.0);
    for _ in 0..10 {
        contract.swap(
            0,
            &accounts(1).to_string(),
            U128(100),
            &accounts(2).to_string(),
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
        testing_env!(context.signer_account_id(accounts(1)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(1),
            U128(2000000),
        );
        testing_env!(context.predecessor_account_id(accounts(2)).build());
        testing_env!(context.signer_account_id(accounts(2)).build());
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
        testing_env!(context
            .signer_account_id(serde_json::from_str(account.as_str()).unwrap())
            .build());
        for _ in 0..10 {
            contract.open_position(0, Some(U128(50)), None, 64.0, 121.0);
        }
        let amount = contract.swap(
            0,
            &accounts(1).to_string(),
            U128(10),
            &accounts(2).to_string(),
        );
        contract.swap(
            0,
            &accounts(2).to_string(),
            amount,
            &accounts(1).to_string(),
        );
    }
    let pool = &contract.pools[0];
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
        testing_env!(context.signer_account_id(accounts(1)).build());
        deposit_tokens(
            &mut context,
            &mut contract,
            serde_json::from_str(account.as_str()).unwrap(),
            accounts(1),
            U128(2000000),
        );
        testing_env!(context.predecessor_account_id(accounts(2)).build());
        testing_env!(context.signer_account_id(accounts(2)).build());
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
        testing_env!(context
            .signer_account_id(serde_json::from_str(account.as_str()).unwrap())
            .build());
        contract.open_position(0, Some(U128(50)), None, 64.0, 121.0);
        let amount = contract.swap(
            0,
            &accounts(1).to_string(),
            U128(10),
            &accounts(2).to_string(),
        );
        contract.swap(
            0,
            &accounts(2).to_string(),
            amount,
            &accounts(1).to_string(),
        );
    }
    let pool = &contract.pools[0];
    assert!(pool.positions.len() == 150);
}
