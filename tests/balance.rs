use mycelium_lab_near_amm::Contract;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{
    json_types::U128,
    test_utils::{accounts, VMContextBuilder},
    testing_env, AccountId,
};
use near_sdk_sim::to_yocto;

fn setup_contract() -> (VMContextBuilder, Contract) {
    let mut context = VMContextBuilder::new();
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    let contract = Contract::new(accounts(0));
    (context, contract)
}

fn deposit_tokens(
    context: &mut VMContextBuilder,
    contract: &mut Contract,
    account_id: AccountId,
    token_id: AccountId,
    amount: U128,
) {
    testing_env!(context
        .predecessor_account_id(token_id)
        .attached_deposit(to_yocto("1"))
        .build());
    contract.ft_on_transfer(account_id.clone(), amount, "".to_string());
}

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
    let balance = contract.get_balance(&accounts(0), &accounts(1)).unwrap();
    assert_eq!(balance, 123456);
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
    let balance = contract.get_balance(&accounts(0), &accounts(1)).unwrap();
    assert_eq!(balance, 30000);
}
