use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

use crate::errors::*;

#[derive(Clone, Serialize, BorshDeserialize, BorshSerialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Position {
    pub id: u128,
    pub owner_id: AccountId,
    pub liquidity: f64,             // L
    pub token0_real_liquidity: f64, // x
    pub token1_real_liquidity: f64, // y
    pub tick_lower_bound_price: i32,
    pub tick_upper_bound_price: i32,
    pub sqrt_lower_bound_price: f64, // p_a
    pub sqrt_upper_bound_price: f64, // p_b
    pub is_active: bool,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            id: 0,
            owner_id: String::new(),
            liquidity: 0.0,
            token0_real_liquidity: 0.0,
            token1_real_liquidity: 0.0,
            tick_lower_bound_price: 0,
            tick_upper_bound_price: 0,
            sqrt_lower_bound_price: 0.0,
            sqrt_upper_bound_price: 0.0,
            is_active: false,
        }
    }
}

impl Position {
    pub fn new(
        id: u128,
        owner_id: AccountId,
        token0_liquidity: Option<u128>,
        token1_liquidity: Option<u128>,
        lower_bound_price: f64,
        upper_bound_price: f64,
        sqrt_price: f64,
    ) -> Position {
        assert!(
            token0_liquidity.is_some() ^ token1_liquidity.is_some(),
            "{}",
            INCORRECT_TOKEN
        );
        let liquidity;
        let x;
        let y;
        let sqrt_lower_bound_price = (lower_bound_price as f64).sqrt();
        let sqrt_upper_bound_price = (upper_bound_price as f64).sqrt();
        if token0_liquidity.is_some() {
            x = token0_liquidity.unwrap() as f64;
            if sqrt_lower_bound_price <= sqrt_price && sqrt_price <= sqrt_upper_bound_price {
                liquidity = get_liquidity_0(x, sqrt_price, sqrt_upper_bound_price);
            } else {
                liquidity = get_liquidity_0(x, sqrt_lower_bound_price, sqrt_upper_bound_price);
            }
            y = calculate_y(
                liquidity,
                sqrt_price,
                sqrt_lower_bound_price,
                sqrt_upper_bound_price,
            );
        } else {
            y = token1_liquidity.unwrap() as f64;
            if sqrt_lower_bound_price <= sqrt_price && sqrt_price <= sqrt_upper_bound_price {
                liquidity = get_liquidity_1(y, sqrt_lower_bound_price, sqrt_price);
            } else {
                liquidity = get_liquidity_1(y, sqrt_lower_bound_price, sqrt_upper_bound_price);
            }
            x = calculate_x(
                liquidity,
                sqrt_price,
                sqrt_lower_bound_price,
                sqrt_upper_bound_price,
            );
        }
        let tick_lower_bound_price = sqrt_price_to_tick(sqrt_lower_bound_price);
        let tick_upper_bound_price = sqrt_price_to_tick(sqrt_upper_bound_price);
        let base = 1.0001 as f64;
        let sqrt_lower_bound_price = base.powf((tick_lower_bound_price / 2) as f64);
        let sqrt_upper_bound_price = base.powf((tick_upper_bound_price / 2) as f64);
        Position {
            id,
            owner_id,
            liquidity,
            token0_real_liquidity: x,
            token1_real_liquidity: y,
            tick_lower_bound_price,
            tick_upper_bound_price,
            sqrt_lower_bound_price,
            sqrt_upper_bound_price,
            is_active: false,
        }
    }

    pub fn refresh(&mut self, sqrt_price: f64) {
        self.token0_real_liquidity = calculate_x(
            self.liquidity,
            sqrt_price,
            self.sqrt_lower_bound_price,
            self.sqrt_upper_bound_price,
        );
        self.token1_real_liquidity = calculate_y(
            self.liquidity,
            sqrt_price,
            self.sqrt_lower_bound_price,
            self.sqrt_upper_bound_price,
        );
    }
}

fn min(first: f64, second: f64) -> f64 {
    if first < second {
        first.clone()
    } else {
        second.clone()
    }
}

fn max(first: f64, second: f64) -> f64 {
    if first > second {
        first.clone()
    } else {
        second.clone()
    }
}

pub fn get_liquidity_0(x: f64, sa: f64, sb: f64) -> f64 {
    x * sa * sb / (sb - sa)
}

pub fn get_liquidity_1(y: f64, sa: f64, sb: f64) -> f64 {
    y / (sb - sa)
}

pub fn get_liquidity(x: f64, y: f64, sp: f64, sa: f64, sb: f64) -> f64 {
    let liquidity;
    if sp <= sa {
        liquidity = get_liquidity_0(x, sa, sb);
    } else if sp < sb {
        let liquidity0 = get_liquidity_0(x, sp, sb);
        let liquidity1 = get_liquidity_1(y, sa, sp);
        liquidity = min(liquidity0, liquidity1)
    } else {
        liquidity = get_liquidity_1(y, sa, sb);
    }
    liquidity
}

pub fn calculate_x(l: f64, sp: f64, sa: f64, sb: f64) -> f64 {
    let sp = max(min(sp, sb), sa);
    l * (sb - sp) / (sp * sb)
}

pub fn calculate_y(l: f64, sp: f64, sa: f64, sb: f64) -> f64 {
    let sp = max(min(sp, sb), sa);
    l * (sp - sa)
}

pub fn calculate_a1(l: f64, sp: f64, _sb: f64, _x: f64, y: f64) -> f64 {
    (sp - y / l).powf(2.0)
}

pub fn calculate_a2(sp: f64, sb: f64, x: f64, y: f64) -> f64 {
    let sa = y / (sb * x) + sp - y / (sp * x);
    sa.powf(2.0)
}

pub fn calculate_b1(l: f64, sp: f64, _sa: f64, x: f64, _y: f64) -> f64 {
    ((l * sp) / (l - sp * x)).powf(2.0)
}

pub fn calculate_b2(sp: f64, sa: f64, x: f64, y: f64) -> f64 {
    let p = sp.powf(2.0);
    (sp * y / ((sa * sp - p) * x + y)).powf(2.0)
}

pub fn tick_to_sqrt_price(tick: i32) -> f64 {
    (1.0001_f64).powf(tick as f64 / 2.0)
}

pub fn sqrt_price_to_tick(sqrt_price: f64) -> i32 {
    sqrt_price.log(1.0001_f64.sqrt()).floor() as i32
}

#[cfg(test)]

mod test {
    use super::min;
    use crate::position::*;
    use crate::{position::max, *};
    #[test]
    fn debug_info() {
        let p = 3227.02_f64;
        let a = 1626.3_f64;
        let b = 4846.3_f64;
        let x = 1_f64;
        let y = 5096.06_f64;
        println!("p = {}, a = {}, b = {}, x = {}, y = {}", p, a, b, x, y);
    }
    #[test]
    fn min_vault() {
        let first = 50_f64;
        let second = 100_f64;
        assert_eq!(min(first, second), 50_f64);
    }
    #[test]
    fn max_vault() {
        let first = 50_f64;
        let second = 100_f64;
        assert_eq!(max(first, second), 100_f64);
    }
    #[test]
    fn get_liquidity_0_test() {
        let sa = 1626.3_f64.powf(0.5);
        let sb = 4846.3_f64.powf(0.5);
        let x = 1_f64;
        let l_0 = get_liquidity_0(x, sa.powf(0.5), sb.powf(0.5)).floor();
        assert_eq!(l_0, 26.0);
        println!("sa = {}, sb = {}, x = {}, l_0 = {}", sa, sb, x, l_0);
    }
    #[test]
    fn get_liquidity_1_test() {
        let sa = 1626.3_f64.powf(0.5);
        let sb = 4846.3_f64.powf(0.5);
        let y = 5096.06_f64;
        let l_1 = get_liquidity_1(y, sa.powf(0.5), sb.powf(0.5)).floor();
        assert_eq!(l_1, 2556.0);
        println!("sa = {}, sb = {}, y = {}, l_1 = {}", sa, sb, y, l_1);
    }
    #[test]
    fn get_liquidity_test() {
        // At sp <= sa ((x * sa * sb)/(sb - sa))
        let mut sp = 1500.02_f64.powf(0.5);
        let mut sa = 3500.3_f64.powf(0.5);
        let mut sb = 1500.3_f64.powf(0.5);
        let mut x = 2_f64;
        let mut y = 5096.06_f64;
        let mut l = get_liquidity(x, y, sp, sa, sb).floor();
        assert_eq!(l, -225.0);
        println!("sp <= sa, l = {}", l);
        // At sp < sb
        // min(get_liquidity_0, get_liquidity_1)
        // get_liquidity_0 = ((x * sa * sb)/(sb - sa))
        // get_liquidity_1 = y /(sb - sa)
        sp = 3227.02_f64.powf(0.5);
        sa = 3000.3_f64.powf(0.5);
        sb = 3800.3_f64.powf(0.5);
        x = 1_f64;
        y = 5096.06_f64;
        l = get_liquidity(x, y, sp, sa, sb).floor();
        assert_eq!(l, 723.0);
        println!("sp < sb, l = {}", l);
        // At sa < sp > sb
        sp = 3600.02_f64.powf(0.5);
        sa = 3500.3_f64.powf(0.5);
        sb = 3000.3_f64.powf(0.5);
        x = 1_f64;
        y = 5096.06_f64;
        l = get_liquidity(x, y, sp, sa, sb).floor();
        assert_eq!(l, -1162.0);
        println!(" sa < sp > sb, l = {}", l);
    }
    #[test]
    fn calculate_x_test() {
        let sp = 3227.02_f64.powf(0.5);
        let sa = 1626.3_f64.powf(0.5);
        let sb = 4846.3_f64.powf(0.5);
        let x = 1_f64;
        let y = 5096.06_f64;
        let l = get_liquidity(x, y, sp, sa, sb);
        let x1 = calculate_x(l, sp, sa, sb);
        assert_eq!(x, 1.00);
        assert!(x == 1.0);
        println!("old x = {}, new x = {}", x, x1);
    }
    #[test]
    fn calculate_y_test() {
        let sp = 3227.02_f64.powf(0.5);
        let sa = 1626.3_f64.powf(0.5);
        let sb = 4846.3_f64.powf(0.5);
        let x = 1_f64;
        let y = 5096.06_f64;
        let l = get_liquidity(x, y, sp, sa, sb);
        let y1 = calculate_y(l, sp, sa, sb);
        assert_eq!(y1.floor(), 5088.0);
        println!("old y = {}, new y = {}", y, y1);
    }
    #[test]
    fn calculate_a1_test() {
        let sp = 3227.02_f64.powf(0.5);
        let a = 1626.3_f64;
        let sa = 1626.3_f64.powf(0.5);
        let sb = 4846.3_f64.powf(0.5);
        let x = 1_f64;
        let y = 5096.06_f64;
        let l = get_liquidity(x, y, sp, sa, sb);
        let a1 = calculate_a1(l, sp, sb, x, y);
        assert_eq!(a1.floor(), 1624.0);
        println!("old a = {}, new a = {}", a, a1);
    }
    #[test]
    fn calculate_a2_test() {
        let sp = 3227.02_f64.powf(0.5);
        let a = 1626.3_f64;
        let sb = 4846.3_f64.powf(0.5);
        let x = 1_f64;
        let y = 5096.06_f64;
        let a2 = calculate_a2(sp, sb, x, y);
        assert_eq!(a2.floor(), 1624.0);
        println!("old a = {}, new a delta = {}", a, a2);
    }
    #[test]
    fn calculate_b1_test() {
        let sp = 3227.02_f64.powf(0.5);
        let sa = 1626.3_f64.powf(0.5);
        let b = 4846.3_f64;
        let sb = 4846.3_f64.powf(0.5);
        let x = 1_f64;
        let y = 5096.06_f64;
        let l = get_liquidity(x, y, sp, sa, sb);
        let b1 = calculate_b1(l, sp, sa, x, y);
        assert_eq!(b1.floor(), 4846.0);
        println!("old b = {}, new b = {}", b, b1);
    }
    #[test]
    fn calculate_b2_test() {
        let sp = 3227.02_f64.powf(0.5);
        let sa = 1626.3_f64.powf(0.5);
        let b = 4846.3_f64;
        let x = 1_f64;
        let y = 5096.06_f64;
        let b2 = calculate_b2(sp, sa, x, y);
        assert_eq!(b2.floor(), 4842.0);
        println!("old b = {}, new b delta = {}", b, b2);
    }
    #[test]
    fn open_position() {
        let position = Position::new(0, String::new(), Some(50), None, 25.0, 121.0, 10.0);
        assert!(position.id == 0, "{}", BAD_POSITION_ID);
        assert!(position.owner_id == String::new(), "{}", NO_VALID_OWNER_ID);
        assert!(
            position.token0_real_liquidity.floor() == 50.0,
            "{}",
            TOKEN0_LIQUIDITY_DOESNT_MATCH
        );
        assert!(
            position.token1_real_liquidity == 27500.0,
            "{}",
            TOKEN1_LIQUIDITY_DOESNT_MATCH
        );
        assert!(position.liquidity == 5500.0, "{}", LIQUIDITY_DOESNT_MATCH);
        assert!(
            position.tick_lower_bound_price == 32190,
            "{}",
            BAD_TICK_LOWER_BOUND_PRICE
        );
        assert!(
            position.tick_upper_bound_price == 47960,
            "{}",
            BAD_TICK_UPPER_BOUND_PRICE
        );
        assert!(
            position.sqrt_lower_bound_price == 4.999908090496346,
            "{}",
            BAD_SQRT_LOWER_BOUND_PRICE
        );
        assert!(
            position.sqrt_upper_bound_price == 10.999833188399927,
            "{}",
            BAD_SQRT_LOWER_BOUND_PRICE
        );
    }

    #[test]
    fn open_position_less_than_lower_bound() {
        let position = Position::new(0, String::new(), Some(50), None, 121.0, 144.0, 10.0);
        assert!(position.id == 0, "{}", BAD_POSITION_ID);
        assert!(position.owner_id == String::new(), "{}", NO_VALID_OWNER_ID);
        assert!(
            position.token0_real_liquidity == 50.0,
            "{}",
            TOKEN0_LIQUIDITY_DOESNT_MATCH
        );
        assert!(
            position.token1_real_liquidity == 0.0,
            "{}",
            TOKEN1_LIQUIDITY_DOESNT_MATCH
        );
        assert!(position.liquidity == 6600.0, "{}", LIQUIDITY_DOESNT_MATCH);
        assert!(
            position.tick_lower_bound_price == 47960,
            "{}",
            BAD_TICK_LOWER_BOUND_PRICE
        );
        assert!(
            position.tick_upper_bound_price == 49700,
            "{}",
            BAD_TICK_UPPER_BOUND_PRICE
        );
        assert!(
            position.sqrt_lower_bound_price == 10.999833188399927,
            "{}",
            BAD_SQRT_LOWER_BOUND_PRICE
        );
        assert!(
            position.sqrt_upper_bound_price == 11.99962930765891,
            "{}",
            BAD_SQRT_LOWER_BOUND_PRICE
        );
    }

    #[test]
    fn open_position_more_than_upper_bound() {
        let position = Position::new(0, String::new(), None, Some(50), 121.0, 144.0, 13.0);
        assert!(position.id == 0, "{}", BAD_POSITION_ID);
        assert!(position.owner_id == String::new(), "{}", NO_VALID_OWNER_ID);
        assert!(
            position.token0_real_liquidity == 0.0,
            "{}",
            TOKEN0_LIQUIDITY_DOESNT_MATCH
        );
        assert!(
            position.token1_real_liquidity == 50.0,
            "{}",
            TOKEN1_LIQUIDITY_DOESNT_MATCH
        );
        assert!(position.liquidity == 50.0, "{}", LIQUIDITY_DOESNT_MATCH);
        assert!(
            position.tick_lower_bound_price == 47960,
            "{}",
            BAD_TICK_LOWER_BOUND_PRICE
        );
        assert!(
            position.tick_upper_bound_price == 49700,
            "{}",
            BAD_TICK_UPPER_BOUND_PRICE
        );
        assert!(
            position.sqrt_lower_bound_price == 10.999833188399927,
            "{}",
            BAD_SQRT_LOWER_BOUND_PRICE
        );
        assert!(
            position.sqrt_upper_bound_price == 11.99962930765891,
            "{}",
            BAD_SQRT_LOWER_BOUND_PRICE
        );
    }
}