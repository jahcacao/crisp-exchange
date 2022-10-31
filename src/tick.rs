use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Tick {
    pub liquiqity_net: f64,
    pub liquidity_gross: f64,
}
