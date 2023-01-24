use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;

use crate::deposit::{BASIS_POINT_BASE, MS_IN_YEAR};

pub type BorrowId = u128;
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Borrow {
    pub owner_id: AccountId,
    pub asset: AccountId,
    pub borrowed: u128,
    pub collateral: u128,
    pub position_id: u128,
    pub pool_id: usize,
    pub last_update_timestamp: u64,
    pub apr: u16,
    pub leverage: Option<u128>,
    pub fees: u128,
    pub liquidation_price: f64,
}

impl Borrow {
    pub fn update_timestamp(&mut self, current_timestamp: u64) {
        self.last_update_timestamp = current_timestamp;
    }

    pub fn calculate_fees(&self, current_timestamp: u64) -> u128 {
        let fees = (self.borrowed as f64)
            * Self::timestamp_difference_to_coefficient(
                current_timestamp - self.last_update_timestamp,
                self.apr,
            );
        fees.round() as u128
    }

    fn timestamp_difference_to_coefficient(timestamp_difference: u64, apr: u16) -> f64 {
        (timestamp_difference as f64 / MS_IN_YEAR as f64)
            * (1_f64 + apr as f64 / BASIS_POINT_BASE as f64)
    }

    pub fn refresh_fees(&mut self, current_timestamp: u64) {
        self.fees += self.calculate_fees(current_timestamp);
    }
}
