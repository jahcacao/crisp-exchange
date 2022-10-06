use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct Pool {
    pub id: u8,
    pub tokens: Vec<AccountId>,
    pub liquidity: Vec<u128>,
}

impl Pool {
    pub fn new(id: u8, token1: AccountId, token2: AccountId) -> Pool {
        Pool {
            id: id,
            tokens: vec![token1, token2],
            liquidity: vec![0, 0],
        }
    }
}
