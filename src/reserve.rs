use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Reserve {
    pub deposited: u128,
    pub borrowed: u128,
    pub total_liquidity: u128,
    pub liquidation_threshold: f64,
    pub loan_to_value: f64,
    pub utilization_rate: f64,
    pub target_utilization_rate: f64,
}

impl Default for Reserve {
    fn default() -> Self {
        Reserve {
            deposited: 0,
            borrowed: 0,
            total_liquidity: 0,
            liquidation_threshold: 0.0,
            loan_to_value: 0.0,
            utilization_rate: 0.0,
            target_utilization_rate: 0.0,
        }
    }
}

impl Reserve {
    pub fn increase_deposit(&mut self, amount: u128) {
        self.deposited += amount;
    }

    pub fn decrease_deposit(&mut self, amount: u128) {
        self.deposited -= amount;
    }

    pub fn calculate_liquidation_threshold(&self) -> f64 {
        0.0
    }

    pub fn refresh_utilization_rate(&mut self) {
        self.utilization_rate = match self.total_liquidity {
            0 => 0.0,
            _ => self.borrowed as f64 / self.total_liquidity as f64,
        }
    }
}
#[cfg(test)]
mod test {

    use crate::reserve::*;

    #[test]
    fn increase_deposit_test() {
        let mut reserve = Reserve::default();
        let new_amount = 500;
        assert!(reserve.deposited == 0);
        reserve.increase_deposit(new_amount);
        assert!(reserve.deposited == 500);
    }
    #[test]
    fn decrease_deposit_test() {
        let mut reserve = Reserve::default();
        let new_amount = 500;
        assert!(reserve.deposited == 0);
        reserve.increase_deposit(new_amount);
        assert!(reserve.deposited == 500);
        reserve.decrease_deposit(200);
        assert!(reserve.deposited == 300);
    }
    #[test]
    fn refresh_utilization_rate_test() {
        let mut reserve = Reserve::default();
        reserve.utilization_rate = 100.0;
        reserve.total_liquidity = 20;
        reserve.borrowed = 50;
        assert!(reserve.utilization_rate == 100.0);
        assert!(reserve.total_liquidity == 20);
        assert!(reserve.borrowed == 50);
        reserve.refresh_utilization_rate();
        assert!(reserve.utilization_rate == 2.5);
    }
}

