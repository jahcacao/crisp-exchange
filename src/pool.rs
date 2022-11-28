use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

use crate::{
    errors::NOT_ENOUGH_LIQUIDITY_IN_POOL,
    position::{sqrt_price_to_tick, tick_to_sqrt_price, Position},
};

#[derive(Clone)]
pub struct CollectedFee {
    pub account_id: AccountId,
    pub amount: f64,
    pub token: AccountId,
}

#[derive(Clone)]
pub struct SwapResult {
    pub amount: f64,
    pub new_liquidity: f64,
    pub new_sqrt_price: f64,
    pub collected_fees: HashMap<u128, CollectedFee>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub token0_locked: u128,
    pub token1_locked: u128,
    pub tick: i32,
    pub positions: Vec<Position>,
    pub protocol_fee: u16,
    pub rewards: u16,
}

impl Pool {
    pub fn new(
        token0: AccountId,
        token1: AccountId,
        price: f64,
        protocol_fee: u16,
        rewards: u16,
    ) -> Pool {
        let tick = sqrt_price_to_tick(price.sqrt());
        Pool {
            token0,
            token1,
            liquidity: 0.0,
            sqrt_price: price.sqrt(),
            token0_locked: 0,
            token1_locked: 0,
            positions: vec![],
            tick,
            protocol_fee,
            rewards,
        }
    }

    pub fn get_swap_result(
        &self,
        token: &AccountId,
        amount: u128,
        direction: SwapDirection,
    ) -> SwapResult {
        if direction == SwapDirection::Return {
            if token == &self.token0 {
                if amount > self.token0_locked {
                    panic!("{}", NOT_ENOUGH_LIQUIDITY_IN_POOL);
                }
            } else {
                if amount > self.token1_locked {
                    panic!("{}", NOT_ENOUGH_LIQUIDITY_IN_POOL);
                }
            }
        }
        let mut collected = 0.0;
        let mut tick = sqrt_price_to_tick(self.sqrt_price);
        let mut price = self.sqrt_price;
        let mut remaining = amount as f64;
        let mut collected_fees: HashMap<u128, CollectedFee> = HashMap::new();
        while remaining > 0.0 {
            let liquidity = self.calculate_liquidity_within_tick(price);
            if liquidity == 0.0 && !self.check_available_liquidity(price, token, direction) {
                panic!("{}", NOT_ENOUGH_LIQUIDITY_IN_POOL);
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
            self.collect_fees(liquidity, price, temp, token, &mut collected_fees);
            collected += temp;
        }
        let liquidity = self.calculate_liquidity_within_tick(price);
        SwapResult {
            amount: collected,
            new_liquidity: liquidity,
            new_sqrt_price: price,
            collected_fees,
        }
    }

    fn collect_fees(
        &self,
        liquidity: f64,
        sqrt_price: f64,
        amount: f64,
        token: &AccountId,
        collected_fees: &mut HashMap<u128, CollectedFee>,
    ) {
        for position in &self.positions {
            if position.is_active(sqrt_price) {
                let share =
                    (position.liquidity / liquidity) * amount * (self.rewards as f64 / 10000.0);
                let old_collected_fee_option = collected_fees.get(&position.id);
                let mut old_share = 0.0;
                if let Some(old_collected_fee) = old_collected_fee_option {
                    old_share = old_collected_fee.amount;
                }
                let collected_fee = CollectedFee {
                    account_id: position.owner_id.clone(),
                    amount: share + old_share,
                    token: self.toggle_token(token),
                };
                collected_fees.insert(position.id, collected_fee);
            }
        }
    }

    fn toggle_token(&self, token: &AccountId) -> AccountId {
        if token == &self.token0 {
            self.token1.to_string()
        } else {
            self.token0.to_string()
        }
    }

    fn check_available_liquidity(
        &self,
        sqrt_price: f64,
        token: &AccountId,
        direction: SwapDirection,
    ) -> bool {
        for position in &self.positions {
            if direction == SwapDirection::Expense && *token == self.token1
                || direction == SwapDirection::Return && *token == self.token0
            {
                // price goes down
                if position.sqrt_upper_bound_price < sqrt_price {
                    return true;
                }
            } else {
                // price goes up
                if position.sqrt_lower_bound_price > sqrt_price {
                    return true;
                }
            }
        }
        false
    }

    fn calculate_liquidity_within_tick(&self, sqrt_price: f64) -> f64 {
        let mut liquidity = 0.0;
        for position in &self.positions {
            if position.is_active(sqrt_price) {
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
        let amount_out;
        if token_out == &self.token1 {
            let new_tick = *tick - 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_in = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            amount_out = (new_sqrt_price - *sqrt_price) * liquidity;
            if amount_out.abs() > *remaining {
                let delta_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price - delta_sqrt_price;
                amount_in = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining -= amount_out.abs();
                *tick -= 1;
            }
        } else {
            let new_tick = *tick + 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_in = (new_sqrt_price - *sqrt_price) * liquidity;
            amount_out = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            if amount_out.abs() > *remaining {
                let delta_reversed_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price / (-delta_reversed_sqrt_price * *sqrt_price + 1.0);
                amount_in = (new_sqrt_price - *sqrt_price) * liquidity;
                *remaining = 0.0;
            } else {
                *remaining -= amount_out.abs();
                *tick += 1;
            }
        }
        *sqrt_price = new_sqrt_price;
        amount_in.abs()
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
        let amount_in;
        if token_in == &self.token1 {
            let new_tick = *tick + 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_in = (new_sqrt_price - *sqrt_price) * liquidity;
            amount_out = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            assert!(new_sqrt_price > *sqrt_price);
            assert!(amount_out < 0.0);
            if amount_in > *remaining {
                let delta_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price + delta_sqrt_price;
                amount_out = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
                assert!(new_sqrt_price > *sqrt_price);
                assert!(amount_out < 0.0);
                *remaining = 0.0;
            } else {
                *remaining -= amount_in.abs();
                *tick += 1;
            }
        } else {
            let new_tick = *tick - 1;
            new_sqrt_price = tick_to_sqrt_price(new_tick);
            amount_in = (1.0 / new_sqrt_price - 1.0 / *sqrt_price) * liquidity;
            amount_out = (new_sqrt_price - *sqrt_price) * liquidity;
            assert!(new_sqrt_price < *sqrt_price);
            assert!(amount_out < 0.0);
            if amount_in > *remaining {
                let delta_reversed_sqrt_price = *remaining / liquidity;
                new_sqrt_price = *sqrt_price / (delta_reversed_sqrt_price * *sqrt_price + 1.0);
                amount_out = (new_sqrt_price - *sqrt_price) * liquidity;
                assert!(new_sqrt_price < *sqrt_price);
                assert!(amount_out < 0.0);
                *remaining = 0.0;
            } else {
                *remaining -= amount_in.abs();
                *tick -= 1;
            }
        }
        *sqrt_price = new_sqrt_price;
        amount_out.abs()
    }

    pub fn get_sqrt_price(&self) -> f64 {
        self.sqrt_price
    }

    pub fn refresh(&mut self, current_timestamp: u64) {
        let mut liquidity = 0.0;
        let mut token0_locked = 0.0;
        let mut token1_locked = 0.0;
        for position in &mut self.positions {
            position.refresh(self.sqrt_price, current_timestamp);
            if position.is_active(self.sqrt_price) {
                liquidity += position.liquidity;
            }
            token0_locked += position.token0_locked;
            token1_locked += position.token1_locked;
        }
        self.liquidity = liquidity;
        self.token0_locked = token0_locked.round() as u128;
        self.token1_locked = token1_locked.round() as u128;
    }

    pub fn open_position(&mut self, position: Position) {
        self.positions.push(position);
    }

    pub fn close_position(&mut self, id: usize) {
        let position = &self.positions[id];
        if position.is_active(self.sqrt_price) {
            self.liquidity -= position.liquidity;
        }
        self.positions.remove(id);
    }

    pub fn apply_swap_result(&mut self, swap_result: &SwapResult) {
        self.liquidity = swap_result.new_liquidity;
        self.sqrt_price = swap_result.new_sqrt_price;
        self.tick = sqrt_price_to_tick(self.sqrt_price);
        for (id, collected_fee) in &swap_result.collected_fees {
            for position in &mut self.positions {
                if &position.id == id {
                    if collected_fee.token == self.token0 {
                        position.fees_earned_token0 += collected_fee.amount.round() as u128;
                    } else {
                        position.fees_earned_token1 += collected_fee.amount.round() as u128;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{pool::SwapDirection, position::sqrt_price_to_tick, *};
    #[test]
    fn pool_get_expense_x() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(50)), None, 1.0, 10000.0, 7.0);
        assert!(position.liquidity == 376.34409850346157);
        pool.open_position(position);
        let result = pool.get_swap_result(&token0, 10, SwapDirection::Expense);
        assert!(result.amount == 601.965597403578);
        assert!(result.new_sqrt_price == 8.599508534336799);
        assert!(result.new_liquidity == 376.34409850346157);
    }

    #[test]
    fn pool_get_expense_y() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(50)), None, 1.0, 10000.0, 7.0);
        assert!(position.liquidity == 376.34409850346157);
        pool.open_position(position);
        let result = pool.get_swap_result(&token1, 10, SwapDirection::Expense);
        assert!(result.amount == 0.20485926166133644);
        assert!(result.new_sqrt_price == 6.973428572309849);
        assert!(result.new_liquidity == 376.34409850346157);
    }

    #[test]
    fn pool_get_return_x() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(50)), None, 1.0, 10000.0, 10.0);
        assert!(position.liquidity.floor() == 555.0);
        pool.open_position(position);
        pool.refresh(0);
        let exp = pool.get_swap_result(&token0, 1, SwapDirection::Return);
        assert!(exp.amount.floor() == 98.0);
        assert!(exp.new_sqrt_price.floor() == 9.0);
        assert!(exp.new_liquidity.floor() == 555.0);
    }

    #[test]
    fn pool_get_return_y1() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(50)), None, 1.0, 10000.0, 10.0);
        assert!(position.liquidity.floor() == 555.0);
        println!("before opening position");
        pool.open_position(position);
        println!("opening position");
        pool.refresh(0);
        println!("refreshed");
        let result = pool.get_swap_result(&token1, 1000, SwapDirection::Return);
        println!("after result");
        assert!(result.amount.floor() == 8.0);
        assert!(result.new_sqrt_price.floor() == 11.0);
        assert!(result.new_liquidity.floor() == 555.0);
    }
    #[test]
    fn pool_get_expense_x_out_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 25.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(10)), None, 20.0, 26.0, 5.0);
        assert_eq!(position.liquidity, 2578.6245298379777);
        pool.open_position(position);
        pool.refresh(0);
        let result = pool.get_swap_result(&token0, 1, SwapDirection::Expense);
        let new_tick = sqrt_price_to_tick(result.new_sqrt_price);
        assert_ne!(new_tick, pool.tick);
    }

    #[test]
    fn pool_get_expense_y_out_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 25.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(10)), None, 20.0, 26.0, 5.0);
        assert_eq!(position.liquidity, 2578.6245298379777);
        pool.open_position(position);
        pool.refresh(0);
        let result = pool.get_swap_result(&token1, 1, SwapDirection::Expense);
        let new_tick = sqrt_price_to_tick(result.new_sqrt_price);
        assert_ne!(new_tick, pool.tick);
    }

    #[test]
    fn pool_get_expense_x_in_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(500)), None, 99.0, 101.0, 10.0);
        assert_eq!(position.liquidity, 1012698.5416276127);
        pool.open_position(position);
        pool.refresh(0);
        let result = pool.get_swap_result(&token0, 5, SwapDirection::Expense);
        let new_tick = sqrt_price_to_tick(result.new_sqrt_price);
        assert_eq!(new_tick, pool.tick);
    }

    #[test]
    fn pool_get_expense_y_in_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(500)), None, 99.0, 101.0, 10.0);
        assert_eq!(position.liquidity, 1012698.5416276127);
        pool.open_position(position);
        pool.refresh(0);
        let exp = pool.get_swap_result(&token1, 1, SwapDirection::Expense);
        let new_tick = sqrt_price_to_tick(exp.new_sqrt_price);
        assert_eq!(new_tick, pool.tick);
    }
    #[test]
    fn pool_get_return_x_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 105.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(5000)), None, 90.0, 110.0, 10.0);
        pool.open_position(position);
        pool.refresh(0);
        println!("pool.token0_locked = {}", pool.token0_locked);
        println!("pool.token1_locked = {}", pool.token1_locked);
        let result = pool.get_swap_result(&token0, 1, SwapDirection::Return);
        let new_tick = sqrt_price_to_tick(result.new_sqrt_price);
        println!("new_tick = {new_tick}");
        println!("pool.tick = {}", pool.tick);
        assert!(new_tick == pool.tick);
    }

    #[test]
    fn pool_get_return_y_within_one_tick() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(500)), None, 99.0, 101.0, 10.0);
        pool.open_position(position);
        pool.refresh(0);
        let exp = pool.get_swap_result(&token1, 1, SwapDirection::Return);
        let new_tick = sqrt_price_to_tick(exp.new_sqrt_price);
        assert!(new_tick == pool.tick);
    }

    #[test]
    #[should_panic(expected = "Not enough liquidity in pool to cover this swap")]
    fn pool_get_return_not_enough_liquidity() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        pool.get_swap_result(&token1, 1000, SwapDirection::Return);
    }

    #[test]
    #[should_panic(expected = "Not enough liquidity in pool to cover this swap")]
    fn pool_get_expense_not_enough_liquidity() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        pool.get_swap_result(&token1, 1000, SwapDirection::Expense);
    }

    #[test]
    #[should_panic(expected = "Not enough liquidity in pool to cover this swap")]
    fn pool_get_amount_many_positions_panic() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        for i in 1..100 {
            let position = Position::new(
                0,
                String::new(),
                Some(U128(i * 100)),
                None,
                100.0 - i as f64,
                100.0 + i as f64,
                10.0,
            );
            pool.open_position(position);
            pool.refresh(0);
        }
        println!("pool.token0_locked = {}", pool.token0_locked);
        println!("pool.token1_locked = {}", pool.token1_locked);
        pool.get_swap_result(&token0, 1000000, SwapDirection::Return);
        pool.get_swap_result(&token1, 1000000, SwapDirection::Expense);
    }

    #[test]
    fn pool_get_amount_many_positions() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        for i in 1..100 {
            let position = Position::new(
                0,
                String::new(),
                Some(U128(i * 100)),
                None,
                100.0 - i as f64,
                100.0 + i as f64,
                10.0,
            );
            pool.open_position(position);
            pool.refresh(0);
        }
        println!("pool.token0_locked = {}", pool.token0_locked);
        println!("pool.token1_locked = {}", pool.token1_locked);
        pool.get_swap_result(&token0, 495000, SwapDirection::Return);
        pool.get_swap_result(&token1, 1000000, SwapDirection::Expense);
    }

    #[test]
    fn pool_apply_swap_result_return() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 100.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(50)), None, 1.0, 10000.0, 10.0);
        assert!(position.liquidity.floor() == 555.0);
        pool.open_position(position);
        pool.refresh(0);
        let result = pool.get_swap_result(&token0, 1, SwapDirection::Return);
        pool.apply_swap_result(&result);
        assert!(pool.sqrt_price.floor() == 9.0);
        assert!(pool.liquidity.floor() == 555.0);
    }

    #[test]
    fn pool_apply_swap_result_expense() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0, 0, 0);
        let position = Position::new(0, String::new(), Some(U128(50)), None, 1.0, 10000.0, 7.0);
        assert!(position.liquidity == 376.34409850346157);
        pool.open_position(position);
        pool.refresh(0);
        let result = pool.get_swap_result(&token1, 10, SwapDirection::Expense);
        pool.apply_swap_result(&result);
        assert!(pool.sqrt_price == 6.973428572309849);
        assert!(pool.liquidity == 376.34409850346157);
    }

    #[test]
    fn pool_fees_expense() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0, 100, 100);
        let position = Position::new(
            0,
            "user.near".to_string(),
            Some(U128(50)),
            None,
            1.0,
            10000.0,
            7.0,
        );
        pool.open_position(position);
        pool.refresh(0);
        let result = pool.get_swap_result(&token1, 10, SwapDirection::Expense);
        let amount = result.amount / 100.0;
        let mut fee = 0.0;
        for (_, collected_fee) in result.collected_fees {
            fee += collected_fee.amount;
        }
        assert!((amount - fee).abs() < 0.00001);
    }

    #[test]
    fn pool_fees_return() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0, 100, 100);
        let position = Position::new(
            0,
            "user.near".to_string(),
            Some(U128(50)),
            None,
            1.0,
            10000.0,
            7.0,
        );
        pool.open_position(position);
        pool.refresh(0);
        let result = pool.get_swap_result(&token1, 10, SwapDirection::Return);
        let amount = result.amount / 100.0;
        let mut fee = 0.0;
        for (_, collected_fee) in result.collected_fees {
            fee += collected_fee.amount;
        }
        assert!((amount - fee).abs() < 0.00001);
    }

    #[test]
    fn pool_fees2() {
        let token0 = "first".to_string();
        let token1 = "second".to_string();
        let mut pool = Pool::new(token0.clone(), token1.clone(), 49.0, 100, 100);
        for _ in 0..9 {
            let position = Position::new(
                0,
                "user.near".to_string(),
                Some(U128(50)),
                None,
                1.0,
                10000.0,
                7.0,
            );
            pool.open_position(position);
            pool.refresh(0);
        }
        let result = pool.get_swap_result(&token1, 10, SwapDirection::Expense);
        let amount = result.amount / 100.0;
        let mut fee = 0.0;
        for (_, collected_fee) in result.collected_fees {
            fee += collected_fee.amount;
        }
        assert!((amount - fee).abs() < 0.00001);
    }
}
