use near_sdk::json_types::U128;
use near_sdk::test_utils::accounts;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;

use crate::common::utils::deposit_tokens;
use crate::common::utils::setup_contract;

mod common;

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
    testing_env!(context.signer_account_id(accounts(1)).build());
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(100000),
    );
    testing_env!(context.signer_account_id(accounts(1)).build());
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
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    testing_env!(context.signer_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(1),
        U128(0),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    testing_env!(context.signer_account_id(accounts(2)).build());
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
    testing_env!(context.signer_account_id(accounts(3)).build());
    let result: u128 = contract
        .swap(
            0,
            &accounts(2).to_string(),
            U128(amount1),
            &accounts(1).to_string(),
        )
        .into();
    let balance1_after: u128 = contract
        .get_balance(&accounts(3).to_string(), &accounts(1).to_string())
        .into();
    let balance2_after: u128 = contract
        .get_balance(&accounts(3).to_string(), &accounts(2).to_string())
        .into();
    let amount2 = result as f64 * 0.98;
    assert!((balance1_after as f64 - amount2).abs() < 20.0);
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
    testing_env!(context.signer_account_id(accounts(0)).build());
    contract.open_position(0, Some(U128(50000)), None, 81.0, 121.0);
    contract.open_position(0, Some(U128(50000)), None, 91.0, 111.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    testing_env!(context.signer_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(3),
        accounts(1),
        U128(100000),
    );
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    testing_env!(context.signer_account_id(accounts(2)).build());
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
    testing_env!(context.signer_account_id(accounts(3)).build());
    let _pool = &contract.pools[0];
    let _result: u128 = contract
        .swap(
            0,
            &accounts(2).to_string(),
            U128(amount1),
            &accounts(1).to_string(),
        )
        .into();
    let _pool = &contract.pools[0];
    let _result: u128 = contract
        .swap(
            0,
            &accounts(1).to_string(),
            U128(99001),
            &accounts(2).to_string(),
        )
        .into();
    let pool = &contract.pools[0];
    let position = pool.positions.get(&0).unwrap();
    assert!(position.fees_earned_token0 == 4);
    assert!(position.fees_earned_token1 == 46522);
    let position = pool.positions.get(&1).unwrap();
    assert!(position.fees_earned_token0 == 6);
    assert!(position.fees_earned_token1 == 46007);
}
