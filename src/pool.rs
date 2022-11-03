use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

use crate::{
    position::{sqrt_price_to_tick, tick_to_sqrt_price, Position},
    tick::Tick,
};

#[derive(Clone, Copy)]
pub struct SwapResult {
    pub amount: f64,
    pub new_liquidity: f64,
    pub new_sqrt_price: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SwapDirection {
    Return,
    Expense,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub token0: AccountId,
    pub token1: AccountId,
    pub liquidity: f64,
    pub sqrt_price: f64,
    pub tick: i32,
    pub ticks_range: HashMap<i32, Tick>,
    pub positions: Vec<Position>,
}

impl Pool {
    pub fn new(token0: AccountId, token1: AccountId, price: f64) -> Pool {
        let tick = sqrt_price_to_tick(price.sqrt());
        Pool {
            token0,
            token1,
            liquidity: 0.0,
            sqrt_price: price.sqrt(),
            positions: vec![],
            tick: tick,
            ticks_range: HashMap::new(),
        }
    }

    pub fn get_swap_result(
        &self,
        token: &AccountId,
        amount: u128,
        direction: SwapDirection,
    ) -> SwapResult {
        let mut collected = 0.0;
        let mut tick = self.tick;
        let mut price = self.sqrt_price;
        let mut remaining = amount as f64;
        while remaining > 0.0 {
            let liquidity = self.calculate_liquidity_within_tick(price);
            if liquidity == 0.0 && !self.check_available_liquidity(price, token, direction) {
                panic!("Not enough liquidity in pool to cover this swap");
            }
            let temp = match direction {
                SwapDirection::Expense => self.get_amount_in_within_tick(
                    &mut tick,
                    &mut price,
                    token,
                    &mut remaining,
                    liquidity,
                ),
                SwapDirection::Return => self.get_amount_out_within_tick(
                    &mut tick,
                    &mut price,
                    token,
                    &mut remaining,
                    liquidity,
                ),
            };
            collected += temp;
        }
        let liquidity = self.calculate_liquidity_within_tick(price);
        SwapResult {
            amount: collected,
            new_liquidity: liquidity,
            new_sqrt_price: price,
        }
    }

    fn check_available_liquidity(
        &self,
        sqrt_price: f64,
        token: &AccountId,
        direction: SwapDirection,
    ) -> bool {
        if direction == SwapDirection::Expense {
            if token.to_string() == self.token1 {
                // price goes down
                for position in &self.positions {
                    if position.sqrt_upper_bound_price < sqrt_price {
                        return true;
                    }
                }
            } else {
                // price goes up
                for position in &self.positions {
                    if position.sqrt_upper_bound_price < sqrt_price {
                        return true;
                    }
                }
            }
        } else {
            if token.to_string() == self.token1 {
                // price goes up
                for position in &self.positions {
                    if position.sqrt_upper_bound_price < sqrt_price {
                        return true;
                    }
                }
            } else {
                // price goes down
                for position in &self.positions {
                    if position.sqrt_upper_bound_price < sqrt_price {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn calculate_liquidity_within_tick(&self, sqrt_price: f64) -> f64 {
        let mut liquidity = 0.0;
        for position in &self.positions {
            if position.sqrt_lower_bound_price <= sqrt_price
                && sqrt_price <= position.sqrt_upper_bound_price
            {
                liquidity += position.liquidity;
            }
        }
        liquidity
    }

    fn get_amount_in_within_tick(
        &self,
        tick: &mut i32,
        sqrt_price: &mut f64,
        token_out: &AccountId,
        remaining: &mut f64,
        liquidity: f64,
    ) -> f64 {
        let mut new_sqrt_price;
        let mut amount_in;
        if token_out == &self.token1 {
            let new_tick = *tick - 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_in = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            let amount_out = (new_sqrt_price - *sqrt_price) * liquidity;
            if -amount_out > *remaining {
                let delta_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price - delta_sqrt_price;
                amount_in = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining += amount_out;
                *tick -= 1;
            }
        } else {
            let new_tick = *tick + 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_in = (new_sqrt_price - *sqrt_price) * liquidity;
            let amount_out = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            if -amount_out > *remaining {
                let delta_reversed_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price / (-delta_reversed_sqrt_price * *sqrt_price + 1.0);
                amount_in = (new_sqrt_price - *sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining += amount_out;
                *tick += 1;
            }
        }
        *sqrt_price = new_sqrt_price;
        return amount_in.abs();
    }

    fn get_amount_out_within_tick(
        &self,
        tick: &mut i32,
        sqrt_price: &mut f64,
        token_in: &AccountId,
        remaining: &mut f64,
        liquidity: f64,
    ) -> f64 {
        let mut new_sqrt_price;
        let mut amount_out;
        if token_in == &self.token1 {
            let new_tick = *tick + 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_out = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            let amount_in = (new_sqrt_price - *sqrt_price) * liquidity;
            if amount_in > *remaining {
                let delta_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price + delta_sqrt_price;
                amount_out = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining -= amount_in;
                *tick += 1;
            }
        } else {
            let new_tick = *tick - 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_out = (new_sqrt_price - *sqrt_price) * liquidity;
            let amount_in = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            if amount_in > *remaining {
                let delta_reversed_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price / (-delta_reversed_sqrt_price * *sqrt_price + 1.0);
                amount_out = (new_sqrt_price - *sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining -= amount_in;
                *tick -= 1;
            }
        }
        *sqrt_price = new_sqrt_price;
        return amount_out.abs();
    }

    pub fn get_sqrt_price(&self) -> f64 {
        self.sqrt_price
    }

    pub fn refresh_liquidity(&mut self) {
        self.liquidity = self.calculate_liquidity_within_tick(self.sqrt_price);
    }

    pub fn open_position(&mut self, position: Position) {
        if position.sqrt_lower_bound_price <= self.sqrt_price
            && position.sqrt_upper_bound_price >= self.sqrt_price
        {
            self.liquidity += position.liquidity;
        }
        self.positions.push(position);
    }

    pub fn close_position(&mut self, id: usize) {
        let position = &self.positions[id];
        if position.sqrt_lower_bound_price <= self.sqrt_price
            && position.sqrt_upper_bound_price >= self.sqrt_price
        {
            self.liquidity -= position.liquidity;
        }
        self.positions.remove(id);
    }

    pub fn apply_swap_result(&mut self, swap_result: SwapResult) {
        self.liquidity = swap_result.new_liquidity;
        self.sqrt_price = swap_result.new_sqrt_price;
    }
}

#[cfg(test)]
mod test {
    use crate::{pool::SwapDirection, position::sqrt_price_to_tick, *};
    #[test]
    fn pool_get_expense_x() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0);
        let position = Position::new(0, String::new(), Some(50), None, 1.0, 10000.0, 7.0);
        assert!(position.liquidity == 376.3440860215054);
        pool.open_position(position);
        let exp = pool.get_swap_result(&token0, 10, SwapDirection::Expense);
        assert!(exp.amount.floor() == 601.0);
        assert!(exp.new_sqrt_price.floor() == 8.0);
        assert!(exp.new_liquidity.floor() == 376.0);
    }

    #[test]
    fn pool_get_expense_y() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0);
        let position = Position::new(0, String::new(), Some(50), None, 1.0, 10000.0, 7.0);
        assert!(position.liquidity == 376.3440860215054);
        pool.open_position(position);
        let exp = pool.get_swap_result(&token1, 10, SwapDirection::Expense);
        assert!(exp.amount.floor() == 0.0);
        assert!(exp.new_sqrt_price.floor() == 6.0);
        assert!(exp.new_liquidity.floor() == 376.0);
    }

    #[test]
    fn pool_get_return_x() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        let position = Position::new(0, String::new(), Some(50), None, 1.0, 10000.0, 10.0);
        assert!(position.liquidity.floor() == 555.0);
        pool.open_position(position);
        let exp = pool.get_swap_result(&token0, 1, SwapDirection::Return);
        assert!(exp.amount.floor() == 98.0);
        assert!(exp.new_sqrt_price.floor() == 9.0);
        assert!(exp.new_liquidity.floor() == 555.0);
    }

    #[test]
    fn pool_get_return_y() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        let position = Position::new(0, String::new(), Some(50), None, 1.0, 10000.0, 10.0);
        assert!(position.liquidity.floor() == 555.0);
        pool.open_position(position);
        let exp = pool.get_swap_result(&token1, 1000, SwapDirection::Return);
        assert!(exp.amount.floor() == 8.0);
        assert!(exp.new_sqrt_price.floor() == 11.0);
        assert!(exp.new_liquidity.floor() == 555.0);
    }

    #[test]
    fn pool_get_return_x_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        let position = Position::new(0, String::new(), Some(500), None, 99.0, 101.0, 10.0);
        pool.open_position(position);
        let exp = pool.get_swap_result(&token0, 1, SwapDirection::Return);
        let new_tick = sqrt_price_to_tick(exp.new_sqrt_price);
        assert!(new_tick == pool.tick);
    }

    #[test]
    fn pool_get_return_y_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        let position = Position::new(0, String::new(), Some(500), None, 99.0, 101.0, 10.0);
        pool.open_position(position);
        let exp = pool.get_swap_result(&token1, 1, SwapDirection::Return);
        let new_tick = sqrt_price_to_tick(exp.new_sqrt_price);
        assert!(new_tick == pool.tick);
    }

    #[test]
    #[should_panic(expected = "Not enough liquidity in pool to cover this swap")]
    fn pool_get_return_not_enough_liquidity() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        pool.get_swap_result(&token1, 1000, SwapDirection::Return);
    }

    #[test]
    #[should_panic(expected = "Not enough liquidity in pool to cover this swap")]
    fn pool_get_expense_not_enough_liquidity() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        pool.get_swap_result(&token1, 1000, SwapDirection::Expense);
    }

    #[test]
    fn pool_get_amount_many_positions() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        for i in 1..100 {
            let position = Position::new(
                0,
                String::new(),
                Some(i * 100),
                None,
                100.0 - i as f64,
                100.0 + i as f64,
                10.0,
            );
            pool.open_position(position);
        }
        pool.get_swap_result(&token0, 1000000, SwapDirection::Return);
        pool.get_swap_result(&token1, 1000000, SwapDirection::Expense);
    }

    #[test]
    fn pool_apply_swap_result_return() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0);
        let position = Position::new(0, String::new(), Some(50), None, 1.0, 10000.0, 10.0);
        assert!(position.liquidity.floor() == 555.0);
        pool.open_position(position);
        let result = pool.get_swap_result(&token0, 1, SwapDirection::Return);
        pool.apply_swap_result(result);
        assert!(pool.sqrt_price.floor() == 9.0);
        assert!(pool.liquidity.floor() == 555.0);
    }

    #[test]
    fn pool_apply_swap_result_expense() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0);
        let position = Position::new(0, String::new(), Some(50), None, 1.0, 10000.0, 7.0);
        assert!(position.liquidity == 376.3440860215054);
        pool.open_position(position);
        let result = pool.get_swap_result(&token1, 10, SwapDirection::Expense);
        pool.apply_swap_result(result);
        assert!(pool.sqrt_price.floor() == 6.0);
        assert!(pool.liquidity.floor() == 376.0);
    }
}
