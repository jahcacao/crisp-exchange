use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Default, Serialize)]
pub struct Pool {
    pub id: u8,
    pub tokens: Vec<AccountId>,
    pub liquidity: Vec<u128>,
}
