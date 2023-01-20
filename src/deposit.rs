use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

pub const MS_IN_YEAR: u64 = 31536000000;
pub const BASIS_POINT_BASE: u16 = 10000;

pub type DepositId = u128;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Deposit {
    pub owner_id: AccountId,
    pub asset: AccountId,
    pub amount: u128,
    pub timestamp: u64,
    pub last_update_timestamp: u64,
    pub apr: u16,
    pub growth: u128,
}

impl Deposit {
    pub fn new(owner_id: AccountId, asset: AccountId, amount: u128) -> Deposit {
        Deposit {
            owner_id,
            asset,
            amount,
            timestamp: 0,
            last_update_timestamp: 0,
            apr: 0,
            growth: 0,
        }
    }
    pub fn update_timestamp(&mut self, current_timestamp: u64) {
        self.last_update_timestamp = current_timestamp;
    }

    pub fn calculate_growth(&self, current_timestamp: u64) -> u128 {
        println!("amount = {}", self.amount);
        println!("current_timestamp = {}", current_timestamp);
        println!(
            "self.last_update_timestamp = {}",
            self.last_update_timestamp
        );
        println!("self.apr = {}", self.apr);
        let growth = (self.amount as f64)
            * Self::timestamp_difference_to_coefficient(
                current_timestamp - self.last_update_timestamp,
                self.apr,
            );
        println!("growth = {growth}");
        growth.round() as u128
    }

    pub fn timestamp_difference_to_coefficient(timestamp_difference: u64, apr: u16) -> f64 {
        (timestamp_difference as f64 / MS_IN_YEAR as f64) * (apr as f64 / BASIS_POINT_BASE as f64)
    }

    pub fn refresh_growth(&mut self, current_timestamp: u64) {
        self.growth += self.calculate_growth(current_timestamp);
        self.last_update_timestamp = current_timestamp;
    }

    pub fn take_growth(&mut self, amount: u128) -> u128 {
        if amount > self.growth {
            println!("amount = {amount} growth = {}", self.growth);
            let result = self.growth;
            self.growth = 0;
            result
        } else {
            println!("amount = {amount} growth = {}", self.growth);
            self.growth -= amount;
            amount
        }
    }
}
#[cfg(test)]
mod test {

    use crate::deposit::*;

    #[test]
    fn timestamp_difference_to_coefficient_test() {
        let asset_token = "wnear".to_string();
        let deposit = Deposit::new(String::new(), asset_token.clone(), 500);
        assert!(deposit.owner_id == String::new(), "{}", "No valid owner id");
        let current_timestamp = 100;
        let last_update_timestamp = 50;
        let timestamp_difference = current_timestamp - last_update_timestamp;
        assert_eq!(timestamp_difference, 50);
        let coefficent =
            Deposit::timestamp_difference_to_coefficient(timestamp_difference, deposit.apr);
        assert_eq!(coefficent, 0.0);
    }

    #[test]
    fn update_timestamp_test() {
        let asset_token = "wnear".to_string();
        let mut deposit = Deposit::new(String::new(), asset_token.clone(), 500);
        assert!(deposit.owner_id == String::new(), "{}", "No valid owner id");
        let current_timestamp = 50;
        assert_eq!(deposit.last_update_timestamp, 0);
        deposit.update_timestamp(current_timestamp);
        assert_eq!(deposit.last_update_timestamp, 50);
    }

    #[test]
    fn calculate_growth_test() {
        let asset_token = "wnear".to_string();
        let mut deposit = Deposit::new(String::new(), asset_token.clone(), 500);
        assert!(deposit.owner_id == String::new(), "{}", "No valid owner id");
        let mut current_timestamp = 20;
        assert!(deposit.last_update_timestamp == 0);
        deposit.update_timestamp(current_timestamp);
        assert!(deposit.last_update_timestamp == 20);
        current_timestamp = 100;
        let coefficentt = Deposit::timestamp_difference_to_coefficient(
            current_timestamp - deposit.last_update_timestamp,
            deposit.apr,
        );
        assert_eq!(coefficentt, 0.0);
        assert!(deposit.amount == 500);
        let growth = deposit.calculate_growth(current_timestamp);
        assert_eq!(growth, 0); // ?
    }

    #[test]
    fn take_growth_test() {
        let asset_token = "wnear".to_string();
        let mut deposit = Deposit::new(String::new(), asset_token.clone(), 500);
        assert!(deposit.owner_id == String::new(), "{}", "No valid owner id");
        assert!(deposit.amount == 500);
        let growth = deposit.take_growth(deposit.amount);
        assert!(growth == 0);
        deposit.growth = 501;
        let growth_1 = deposit.take_growth(deposit.amount);
        assert_eq!(growth_1, 500);
    }

    #[test]
    fn check_growth() {
        let mut deposit = Deposit::new(String::new(), String::new(), 100);
        deposit.apr = 500;
        let index = Deposit::timestamp_difference_to_coefficient(MS_IN_YEAR, 500);
        assert_eq!(index, 0.05);
        let growth = deposit.calculate_growth(MS_IN_YEAR);
        assert_eq!(growth, 5);
        deposit.refresh_growth(MS_IN_YEAR);
        assert_eq!(deposit.growth, 5);
    }
}
