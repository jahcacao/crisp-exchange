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
    contract.create_pool(accounts(0).to_string(), accounts(1).to_string(), 100.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.token0 == accounts(0).to_string());
    assert!(pool.token1 == accounts(1).to_string());
    assert!(pool.liquidity == 0.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions == vec![]);
    assert!(pool.sqrt_price == 10.0);
}

#[test]
fn open_position_is_correct() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(50),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 50);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(27500),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 27500);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(50), None, 25.0, 121.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 5500.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 0);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 0);
}

#[test]
fn open_position_less_than_lower_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
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
    contract.open_position(0, Some(50), None, 121.0, 144.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 1950);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
}

#[test]
fn open_position_more_than_upper_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
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
    contract.open_position(0, None, Some(50), 64.0, 81.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 2950);
}

#[test]
fn open_two_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 121.0);
    contract.open_position(0, Some(100), None, 49.0, 144.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 6025.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 2);
}

#[test]
fn open_three_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 121.0);
    contract.open_position(0, Some(100), None, 49.0, 144.0);
    contract.open_position(0, None, Some(150), 81.0, 169.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 6175.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 3);
}

#[test]
fn open_ten_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 121.0);
    contract.open_position(0, Some(100), None, 49.0, 144.0);
    contract.open_position(0, None, Some(150), 81.0, 169.0);
    contract.open_position(0, Some(200), None, 110.0, 121.0);
    contract.open_position(0, None, Some(250), 49.0, 99.0);
    contract.open_position(0, Some(300), None, 149.0, 154.0);
    contract.open_position(0, None, Some(350), 81.0, 99.0);
    contract.open_position(0, Some(100), None, 49.0, 144.0);
    contract.open_position(0, None, Some(50), 64.0, 121.0);
    contract.open_position(0, Some(500), None, 120.0, 130.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 12200.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 10);
}

#[test]
fn close_position() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 121.0);
    contract.close_position(0, 0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 0);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
}

#[test]
fn close_two_position() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(100), None, 49.0, 144.0);
    contract.open_position(0, Some(100), None, 49.0, 144.0);
    contract.close_position(0, 1);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 6000.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    contract.close_position(0, 0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 0);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000000);
}

#[test]
fn get_expense_within_single_tick() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000000000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000000000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000000000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000000000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(10000000), None, 81.0, 121.0);
    let pool = &contract.get_pool(0).unwrap();
    let position = &pool.positions[0];
    println!(
        "position: token0 = {} token1 = {}",
        position.token0_real_liquidity, position.token1_real_liquidity
    );
    println!("pool liquidity = {}", pool.liquidity);
    let exp1 = contract.get_expense(0, &accounts(1).to_string(), 1);
    let exp2 = contract.get_expense(0, &accounts(2).to_string(), 10);
    println!("token0 amount 100 = {}\n", exp1);
    println!("token1 amount 100 = {}\n", exp2);
    assert!(exp1 == 99.99999903698154); // new price =  exp1 =
    assert!(exp2 == 0.10000000966181588); // new price =  exp2 =
}
