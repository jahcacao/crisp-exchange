use near_sdk::json_types::ValidAccountId;
use near_sdk::json_types::U128;
use near_sdk::test_utils::accounts;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;

use crate::common::utils::deposit_tokens;
use crate::common::utils::setup_contract;

mod common;

#[test]
fn liquidate() {
    let (mut context, mut contract) = setup_contract();
    let alice = ValidAccountId::try_from("john.near").unwrap();
    contract.create_reserve(&accounts(2).into());
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.signer_account_id(alice.clone()).build());
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        alice.clone(),
        accounts(1),
        U128(99999999999999999),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        alice.clone(),
        accounts(2),
        U128(99999999999999999),
    );
    testing_env!(context
        .predecessor_account_id(alice.clone())
        .attached_deposit(1)
        .build());
    let pool = &contract.pools[0];
    assert_eq!(pool.positions.len(), 0);
    contract.open_position(0, Some(U128(1)), None, 99.0, 101.0);
    contract.open_position(0, None, Some(U128(100)), 50.0, 101.0);
    contract.create_deposit(&accounts(2).into(), U128::from(1000000000000));
    contract.supply_collateral_and_borrow_simple(0, 0);
    let h = contract.get_borrow_health_factor(0);
    assert_eq!((h * 100.0).round(), 125.0);
    let list = contract.get_liquidation_list();
    assert!(list.is_empty());
    for _ in 0..2 {
        contract.swap(
            0,
            &accounts(1).to_string(),
            U128(1),
            &accounts(2).to_string(),
        );
    }
    let balance1_before = contract.get_balance(&"john.near".to_string(), &accounts(1).to_string());
    let balance2_before = contract.get_balance(&"john.near".to_string(), &accounts(2).to_string());
    testing_env!(context.build());
    contract.liquidate(0);
    let balance1_after = contract.get_balance(&"john.near".to_string(), &accounts(1).to_string());
    let balance2_after = contract.get_balance(&"john.near".to_string(), &accounts(2).to_string());
    assert_eq!(balance1_before, balance1_after);
    assert!(balance2_before.0 > balance2_after.0);
}

#[test]
fn get_liquidation_list() {
    let (mut context, mut contract) = setup_contract();
    let alice = ValidAccountId::try_from("john.near").unwrap();
    contract.create_reserve(&accounts(2).into());
    contract.create_pool(
        accounts(1).to_string(),
        accounts(2).to_string(),
        100.0,
        0,
        0,
    );
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    testing_env!(context.signer_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        alice.clone(),
        accounts(1),
        U128(99999999999999999),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    testing_env!(context.signer_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        alice.clone(),
        accounts(2),
        U128(99999999999999999),
    );
    testing_env!(context
        .predecessor_account_id(alice.clone())
        .attached_deposit(1)
        .build());
    testing_env!(context.signer_account_id(alice.clone()).build());
    let pool = &contract.pools[0];
    assert_eq!(pool.positions.len(), 0);
    contract.open_position(0, Some(U128(1)), None, 99.0, 101.0);
    contract.open_position(0, None, Some(U128(100)), 50.0, 101.0);
    contract.create_deposit(&accounts(2).into(), U128::from(1000000000000));
    contract.supply_collateral_and_borrow_simple(0, 0);
    let h = contract.get_borrow_health_factor(0);
    assert_eq!((h * 100.0).round(), 125.0);
    let list = contract.get_liquidation_list();
    assert!(list.is_empty());
    for _ in 0..2 {
        contract.swap(
            0,
            &accounts(1).to_string(),
            U128(1),
            &accounts(2).to_string(),
        );
    }
    let list = contract.get_liquidation_list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0], 0);
}

#[test]
fn return_collateral_and_repay() {
    let alice = ValidAccountId::try_from("john.near").unwrap();
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(2).into());
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
    testing_env!(context
        .signer_account_id(alice.clone())
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.create_deposit(&accounts(2).into(), U128::from(100000));
    let borrowed = contract.supply_collateral_and_borrow_simple(0, 0);
    testing_env!(context.predecessor_account_id(alice.clone()).build());
    let _token = contract.tokens_by_id.get(&"0".to_string()).unwrap();
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
fn supply_collateral_and_borrow_leveraged() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(1).into());
    contract.create_reserve(&accounts(2).into());
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
    testing_env!(context
        .signer_account_id(accounts(0))
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    let pool = &contract.pools[0];
    let position = pool.positions.get(&0).unwrap();
    let total_locked = position.total_locked as u128;
    contract.create_deposit(&accounts(1).into(), U128::from(100000));
    contract.create_deposit(&accounts(2).into(), U128::from(100000));
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
    assert_eq!(borrow.last_update_timestamp, 0);
    assert_eq!(borrow.apr, 1000);
    assert_eq!(borrow.leverage, Some(leverage));
    assert_eq!(borrow.fees, 0);
    let token = contract.tokens_by_id.get(&"0".to_string()).unwrap();
    assert_eq!(token.owner_id, context.context.current_account_id);
}

#[test]
fn supply_collateral_and_borrow_simple_should_work() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(2).into());
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
    testing_env!(context
        .signer_account_id(accounts(0))
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.create_deposit(&accounts(2).into(), U128::from(100000));
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
    assert_eq!(borrow.last_update_timestamp, 0);
    assert_eq!(borrow.apr, 1000);
    assert_eq!(borrow.leverage, None);
    assert_eq!(borrow.fees, 0);
    let token = contract.tokens_by_id.get(&"0".to_string()).unwrap();
    assert_eq!(token.owner_id, context.context.current_account_id);
}

#[should_panic(
    expected = "You want to borrow 26004 of charlie but only 10 is available in reserve"
)]
#[test]
fn supply_collateral_and_borrow_simple_not_enough_reserves() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(2).into());
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
    testing_env!(context
        .signer_account_id(accounts(0))
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.create_deposit(&accounts(2).into(), U128::from(10));
    contract.supply_collateral_and_borrow_simple(0, 0);
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
    testing_env!(context
        .signer_account_id(accounts(0))
        .attached_deposit(1)
        .build());
    contract.open_position(0, Some(U128(50)), None, 25.0, 121.0);
    contract.supply_collateral_and_borrow_simple(0, 0);
}

#[test]
fn get_account_deposits() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(&accounts(1).into(), U128::from(10));
    contract.create_deposit(&accounts(1).into(), U128::from(20));
    contract.create_deposit(&accounts(1).into(), U128::from(30));
    contract.create_deposit(&accounts(1).into(), U128::from(40));
    let _deposits = contract.get_account_deposits(&accounts(0).to_string());
}

#[test]
fn take_deposit_growth() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(&accounts(1).into(), U128::from(100));
    let context = context.block_timestamp(31536000000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    context.block_index(31536000);
    context.epoch_height(31536000);
    contract.refresh_deposits_growth();
    let deposit = contract.deposits.get(&0).unwrap();
    assert_eq!(deposit.growth, 5);
    let growth = contract.take_deposit_growth(0, U128::from(10));
    assert_eq!(growth, 5.into());
    let balance = contract.get_balance(&accounts(0).to_string(), &accounts(1).to_string());
    assert_eq!(balance, U128::from(5));
}

#[test]
fn refresh_deposit_growth() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(300),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(&accounts(1).into(), U128::from(100));
    contract.create_deposit(&accounts(1).into(), U128::from(200));
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
fn create_deposit2() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(1).into());
    contract.create_reserve(&accounts(2).into());
    contract.create_reserve(&accounts(3).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(100),
    );
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(3),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(&accounts(1).into(), U128::from(10));
    contract.create_deposit(&accounts(1).into(), U128::from(20));
    contract.create_deposit(&accounts(1).into(), U128::from(30));
    contract.create_deposit(&accounts(2).into(), U128::from(1));
    contract.create_deposit(&accounts(2).into(), U128::from(2));
    contract.create_deposit(&accounts(2).into(), U128::from(3));
    contract.create_deposit(&accounts(3).into(), U128::from(50));
    contract.create_deposit(&accounts(3).into(), U128::from(10));
    contract.create_deposit(&accounts(3).into(), U128::from(20));
    let deposit = contract.deposits.get(&0).unwrap();
    assert_eq!(deposit.owner_id, accounts(0).to_string());
    assert_eq!(deposit.asset, accounts(1).to_string());
    assert_eq!(deposit.amount, 10);
    assert_eq!(deposit.timestamp, 0);
    assert_eq!(deposit.last_update_timestamp, 0);
    assert_eq!(deposit.apr, 500);
    assert_eq!(deposit.growth, 0);

    let _deposits = contract.get_account_deposits(&accounts(0).to_string());
    // assert_eq!(*deposits.get(&accounts(1).to_string()).unwrap(), 60);
    // assert_eq!(*deposits.get(&accounts(2).to_string()).unwrap(), 6);
    // assert_eq!(*deposits.get(&accounts(3).to_string()).unwrap(), 80);
}

#[test]
fn close_deposit() {
    let (mut context, mut contract) = setup_contract();
    contract.create_reserve(&accounts(1).into());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100),
    );
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.create_deposit(&accounts(1).into(), U128::from(100));
    contract.deposits.get(&0).unwrap();
    contract.close_deposit(0);
    assert!(contract.deposits.is_empty());
}

#[test]
fn create_reserve() {
    let (mut _context, mut contract) = setup_contract();
    assert!(contract.reserves.is_empty());
    contract.create_reserve(&"usdt.testnet".to_string());
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
    contract.create_reserve(&"usdt.testnet".to_string());
    contract.create_deposit(&"usn.testnet".to_string(), U128::from(100));
}
