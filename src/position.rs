use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
};

#[derive(Clone, Serialize, BorshDeserialize, BorshSerialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Position {
    pub liquidity: f64,     // L
    pub token0_real_liquidity: u128, // x
    pub token1_real_liquidity: u128, // y
    pub sqrt_lower_bound_price: f64, // p_a
    pub sqrt_upper_bound_price: f64, // p_b
    pub is_active: bool,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            liquidity: 0.0,
            token0_real_liquidity: 0,
            token1_real_liquidity: 0,
            sqrt_lower_bound_price: 0.0,
            sqrt_upper_bound_price: 0.0,
            is_active: false,
        }
    }
}

impl Position {
    pub fn new(
        token0_liquidity: u128,
        token1_liquidity: u128,
        lower_bound_price: u128,
        upper_bound_price: u128,
    ) -> Position {
        let liquidity = ((token0_liquidity as f64) * (token1_liquidity as f64)).sqrt();
        let sqrt_lower_bound_price = (lower_bound_price as f64).sqrt();
        let sqrt_upper_bound_price = (upper_bound_price as f64).sqrt();
        Position {
            liquidity,
            token0_real_liquidity: 0,
            token1_real_liquidity: 0,
            sqrt_lower_bound_price,
            sqrt_upper_bound_price,
            is_active: false,
        }
    }

    pub fn refresh(&mut self, sqrt_price: f64) {
        if sqrt_price > self.sqrt_upper_bound_price {
            println!("too high");
            self.token0_real_liquidity = 0;
            self.token1_real_liquidity = (self.liquidity
                * (self.sqrt_upper_bound_price - self.sqrt_lower_bound_price))
                as u128;
            self.is_active = false;
        } else if sqrt_price < self.sqrt_lower_bound_price {
            println!("too low");
            self.token0_real_liquidity = (self.liquidity
                * (self.sqrt_upper_bound_price - self.sqrt_lower_bound_price)
                / (self.sqrt_upper_bound_price * self.sqrt_lower_bound_price))
                as u128;
            self.token1_real_liquidity = 0;
            self.is_active = false;
        } else {
            println!("normal price");
            self.token0_real_liquidity =
                (self.liquidity * (self.sqrt_upper_bound_price - sqrt_price)
                    / (self.sqrt_upper_bound_price * sqrt_price)) as u128;
            self.token1_real_liquidity =
                (self.liquidity * (sqrt_price - self.sqrt_lower_bound_price)) as u128;
            self.is_active = true;
        }
    }
}
