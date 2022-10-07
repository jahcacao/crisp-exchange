use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
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

    pub fn add_liquidity(&mut self, token: AccountId, amount: u128) {
        if token == self.tokens[0] {
            self.liquidity[0] += amount;
        } else if token == self.tokens[1] {
            self.liquidity[1] += amount;
        } else {
            panic!("Bad token");
        }
    }
}
