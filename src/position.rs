use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::Serialize,
    AccountId,
};

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
        assert!(token0_liquidity.is_some() ^ token1_liquidity.is_some());
        let liquidity;
        let x;
        let y;
        let sqrt_lower_bound_price = (lower_bound_price as f64).sqrt();
        let sqrt_upper_bound_price = (upper_bound_price as f64).sqrt();
        if token0_liquidity.is_some() {
            assert!(
                sqrt_price <= sqrt_upper_bound_price,
                "Wrong token amount chosen"
            );
            x = token0_liquidity.unwrap() as f64;
            liquidity = get_liquidity_0(x, sqrt_price, sqrt_upper_bound_price);
            y = calculate_y(
                liquidity,
                sqrt_price,
                sqrt_lower_bound_price,
                sqrt_upper_bound_price,
            );
        } else {
            assert!(
                sqrt_price >= sqrt_lower_bound_price,
                "Wrong token amount chosen"
            );
            y = token1_liquidity.unwrap() as f64;
            liquidity = get_liquidity_1(y, sqrt_lower_bound_price, sqrt_price);
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
        let liquidity0 = get_liquidity_0(x, sa, sb);
        let liquidity1 = get_liquidity_1(y, sa, sb);
        liquidity = min(liquidity0, liquidity1)
    } else {
        liquidity = get_liquidity_1(y, sa, sb);
    }
    liquidity
}

pub fn calculate_x(L: f64, sp: f64, sa: f64, sb: f64) -> f64 {
    let sp = max(min(sp, sb), sa);
    L * (sb - sp) / (sp * sb)
}

pub fn calculate_y(L: f64, sp: f64, sa: f64, sb: f64) -> f64 {
    let sp = max(min(sp, sb), sa);
    L * (sp - sa)
}

pub fn calculate_a1(L: f64, sp: f64, sb: f64, x: f64, y: f64) -> f64 {
    (sp - y / L).powf(2.0)
}

pub fn calculate_a2(sp: f64, sb: f64, x: f64, y: f64) -> f64 {
    let sa = y / (sb * x) + sp - y / (sp * x);
    sa.powf(2.0)
}

pub fn calculate_b1(L: f64, sp: f64, sa: f64, x: f64, y: f64) -> f64 {
    ((L * sp) / (L - sp * x)).powf(2.0)
}

pub fn calculate_b2(sp: f64, sa: f64, x: f64, y: f64) -> f64 {
    let P = sp.powf(2.0);
    (sp * y / ((sa * sp - P) * x + y)).powf(2.0)
}

pub fn tick_to_price(tick: i32) -> f64 {
    (1.0001_f64).powf(tick as f64)
}

pub fn sqrt_price_to_tick(sqrt_price: f64) -> i32 {
    sqrt_price.log(1.0001_f64.sqrt()).floor() as i32
}

#[cfg(test)]
mod test {
    use crate::*;
    #[test]
    fn open_position() {
        let position = Position::new(0, String::new(), Some(50), None, 25.0, 121.0, 10.0);
        assert!(position.id == 0);
        assert!(position.owner_id == String::new());
        assert!(position.token0_real_liquidity.floor() == 50.0);
        assert!(position.token1_real_liquidity == 27500.0);
        assert!(position.liquidity == 5500.0);
        assert!(position.tick_lower_bound_price == 32190);
        assert!(position.tick_upper_bound_price == 47960);
        assert!(position.sqrt_lower_bound_price == 4.999908090496346);
        assert!(position.sqrt_upper_bound_price == 10.999833188399927);
    }

    #[test]
    fn open_position_less_than_lower_bound() {
        let position = Position::new(0, String::new(), Some(50), None, 121.0, 144.0, 10.0);
        assert!(position.id == 0);
        assert!(position.owner_id == String::new());
        assert!(position.token0_real_liquidity == 50.0);
        assert!(position.token1_real_liquidity == 0.0);
        assert!(position.liquidity == 3000.0);
        assert!(position.tick_lower_bound_price == 47960);
        assert!(position.tick_upper_bound_price == 49700);
        assert!(position.sqrt_lower_bound_price == 10.999833188399927);
        assert!(position.sqrt_upper_bound_price == 11.99962930765891);
    }

    #[test]
    fn open_position_more_than_upper_bound() {
        let position = Position::new(0, String::new(), None, Some(50), 121.0, 144.0, 13.0);
        assert!(position.id == 0);
        assert!(position.owner_id == String::new());
        assert!(position.token0_real_liquidity == 0.0);
        assert!(position.token1_real_liquidity == 50.0);
        assert!(position.liquidity == 25.0);
        assert!(position.tick_lower_bound_price == 47960);
        assert!(position.tick_upper_bound_price == 49700);
        assert!(position.sqrt_lower_bound_price == 10.999833188399927);
        assert!(position.sqrt_upper_bound_price == 11.99962930765891);
    }

    #[should_panic(expected = "Wrong token amount chosen")]
    #[test]
    fn open_position_less_than_lower_bound_wrong_token() {
        let _position = Position::new(0, String::new(), Some(50), None, 121.0, 144.0, 13.0);
    }

    #[should_panic(expected = "Wrong token amount chosen")]
    #[test]
    fn open_position_more_than_upper_bound_wrong_token() {
        let _position = Position::new(0, String::new(), None, Some(50), 121.0, 144.0, 10.0);
    }
}
