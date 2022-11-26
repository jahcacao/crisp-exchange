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
    assert!(pool.positions == vec![]);
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
        U128(27500),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(27500));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity == 5500.0);
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
    assert!(pool.liquidity == 6025.0);
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
    assert!(pool.liquidity == 6175.0);
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
    assert!(pool.liquidity == 12200.0);
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
    assert!(pool.liquidity == 6000.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    contract.close_position(0, 0);
    let pool = contract.get_pool(0);
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 0);
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
        U128(20000000000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128(20000000000));
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000000000),
    );
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance, U128(30000000000));
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(10000000)), None, 81.0, 121.0);
    let exp1 = contract.get_expense(0, &accounts(1).to_string(), U128(1));
    let exp2 = contract.get_expense(0, &accounts(2).to_string(), U128(1));
    assert!(exp1 == U128(100));
    assert!(exp2 == U128(0));
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
        U128(11000000),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(100000));
    assert!(balance2_before == U128(0));
    let amount1 = 100000;
    let amount2 = contract.swap_in(
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
        U128(11100000),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(0));
    assert!(balance2_before == U128(100000));
    let amount1 = 100000;
    let amount2 = contract.swap_in(
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
fn swap_out_token0() {
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
        U128(101000),
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
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(1000));
    assert!(balance2_before == U128(0));
    let amount1 = 100000;
    contract.swap_out(
        0,
        accounts(1).to_string(),
        U128(amount1),
        accounts(2).to_string(),
    );
    let balance1_after = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_after == U128(0));
    assert!(balance2_after == U128(amount1));
}

#[test]
fn swap_out_token1() {
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
        U128(22000000),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(100000)), None, 81.0, 121.0);
    let balance1_before = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(0));
    assert!(balance2_before == U128(11000000));
    let amount1 = 100000;
    contract.swap_out(
        0,
        accounts(2).to_string(),
        U128(amount1),
        accounts(1).to_string(),
    );
    let balance1_after = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    assert_eq!(balance1_after, U128(amount1));
    assert_eq!(balance2_after, U128(0));
}

#[test]
fn fee_test_out() {
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
        U128(11220000),
    );
    let balance1_before = contract.get_balance(&accounts(3).to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&accounts(3).to_string(), &accounts(2).to_string());
    assert!(balance1_before == U128(0));
    assert!(balance2_before == U128(11220000));
    let amount1 = 100000;
    testing_env!(context.predecessor_account_id(accounts(3)).build());
    contract.swap_out(
        0,
        accounts(2).to_string(),
        U128(amount1),
        accounts(1).to_string(),
    );
    let balance1_after = contract.get_balance(&accounts(3).to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&accounts(3).to_string(), &accounts(2).to_string());
    assert_eq!(balance1_after, U128(amount1));
    assert_eq!(balance2_after, U128(0));
    let balance1_lp_after =
        contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    let balance2_lp_after =
        contract.get_balance(&accounts(0).to_string(), &accounts(2).to_string());
    let balance2_before: u128 = balance2_before.into();
    let amount2 = (balance2_before as f64 / 1.02) * 0.01;
    assert!(balance1_lp_after == U128(0));
    let balance2_lp_after: u128 = balance2_lp_after.into();
    assert!((balance2_lp_after as f64 - amount2).abs() < 100.0);
}

#[test]
fn fee_test_in() {
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
        .swap_in(
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
    // let _pool = &contract.pools[0];
    // println!("pool.liquidity = {}", pool.liquidity);
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
    // println!("pool.positions[0].liquidity = {}", pool.positions[0].liquidity);
    // println!("pool.positions[1].liquidity = {}", pool.positions[1].liquidity);
    // println!("pool.liquidity = {}", pool.liquidity);
    let _result: u128 = contract
        .swap_in(
            0,
            accounts(2).to_string(),
            U128(amount1),
            accounts(1).to_string(),
        )
        .into();
    let _pool = &contract.pools[0];
    // println!("pool.positions[0].liquidity = {}", pool.positions[0].liquidity);
    // println!("pool.positions[1].liquidity = {}", pool.positions[1].liquidity);
    // println!("pool.liquidity = {}", pool.liquidity);
    // println!("pool.sqrt_price = {}", pool.sqrt_price);
    let _result: u128 = contract
        .swap_in(
            0,
            accounts(1).to_string(),
            U128(100000),
            accounts(2).to_string(),
        )
        .into();
    let pool = &contract.pools[0];
    // println!("pool.positions[0].liquidity = {} is_active = {}", pool.positions[0].liquidity, pool.positions[0].is_active);
    // println!("pool.positions[1].liquidity = {} is_active = {}", pool.positions[1].liquidity, pool.positions[1].is_active);
    // println!("pool.liquidity = {}", pool.liquidity);
    // println!("pool.sqrt_price = {}", pool.sqrt_price);
    // println!("pool.positions[0].fees_earned_token0 = {}", pool.positions[0].fees_earned_token0);
    // println!("pool.positions[0].fees_earned_token1 = {}", pool.positions[0].fees_earned_token1);
    // println!("pool.positions[1].fees_earned_token0 = {}", pool.positions[1].fees_earned_token0);
    // println!("pool.positions[1].fees_earned_token1 = {}", pool.positions[1].fees_earned_token1);
    assert!(pool.positions[0].fees_earned_token0 == 3);
    assert!(pool.positions[0].fees_earned_token1 == 47382);
    assert!(pool.positions[1].fees_earned_token0 == 6);
    assert!(pool.positions[1].fees_earned_token1 == 45979);
}
