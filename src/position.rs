use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

#[derive(Clone, Serialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Position {
    pub primary_token: AccountId,
    pub lower_price: u128,
    pub upper_price: u128,
    pub liquidity: Vec<u128>,
    pub is_active: bool,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            primary_token: String::new(),
            lower_price: 0,
            upper_price: 0,
            liquidity: vec![],
            is_active: false,
        }
    }
}

impl Position {
    pub fn refresh_activity(&mut self, price: u128) {
        if price >= self.lower_price && price <= self.upper_price {
            self.is_active = true;
        } else {
            self.is_active = false;
        }
    }
}
