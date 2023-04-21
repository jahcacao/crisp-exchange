use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
    AccountId,
};

/// Single swap action.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    pub pool_id: usize,
    pub token_in: AccountId,
    pub amount_in: U128,
    pub token_out: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MultihopeSwapAction {
    pub token_in: AccountId,
    pub amount_in: U128,
    pub token_out: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OpenPositionAction {
    pub request_id: usize,
    pub pool_id: usize,
    pub token0_liquidity: Option<U128>,
    pub token1_liquidity: Option<U128>,
    pub lower_bound_price: f64,
    pub upper_bound_price: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AddLiquidityAction {
    pub pool_id: usize,
    pub position_id: u128,
    pub token0_liquidity: Option<U128>,
    pub token1_liquidity: Option<U128>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CreateDepositAction {
    pub asset: AccountId,
    pub amount: U128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ReturnCollateralAndRepayAction {
    pub borrow_id: u128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidateAction {
    pub borrow_id: u128,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawAction {
    pub token: AccountId,
    pub amount: U128,
}

/// Single action. Allows to execute sequence of various actions initiated by an account.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    Swap(SwapAction),
    Withdraw(WithdrawAction),
    MultihopeSwap(MultihopeSwapAction),
    OpenPosition(OpenPositionAction),
    AddLiquidity(AddLiquidityAction),
    CreateDeposit(CreateDepositAction),
    ReturnCollateralAndRepay(ReturnCollateralAndRepayAction),
    Liquidate(LiquidateAction),
}
