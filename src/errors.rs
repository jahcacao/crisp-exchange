use near_sdk::AccountId;

pub const BAD_POOL_ID: &str = "Bad pool_id";

pub const NFT0: &str = "NFT not found";
pub const NFT1: &str = "Predecessor must be the token owner.";
pub const NFT2: &str = "Requires attached deposit of exactly 1 yoctoNEAR";
pub const NFT3: &str = "Token should be owned by the sender";
pub const _NFT4: &str = "Unauthorized";
pub const NFT5: &str = "Sender is not approved account";
pub const NFT6: &str = "The token owner and the receiver should be different";
pub const NFT7: &str = "Token already exists";
pub const NFT8: &str = "Market cannot payout to that many receivers";

pub const BAL0: &str = "Token has not been deposited";
pub const BAL1: &str = "Account is not registered";

pub const SWP0: &str = "Route does not exist!";
pub const SWP1: &str = "Not enough liquidity in pool to cover this swap";

pub const PST0: &str = "Position not found";
pub const PST1: &str = "token0 liqudity cannot be 0";
pub const PST2: &str = "send token1 liquidity instead of token0";
pub const PST3: &str = "token1 liqudity cannot be 0";
pub const PST4: &str = "send token0 liquidity instead of token1";
pub const PST5: &str = "Incorrect token";

pub const RSR0: &str = "Reserve not found";

pub const DPS0: &str = "Deposit not found";
pub const DPS1: &str = "You do not own this deposit";

pub const BRR0: &str = "Borrow not found";

pub fn withdraw_error(token: &AccountId, amount: u128, amount_actual: u128) -> String {
    format!(
        "You want to withdraw {} of {} but only have {}",
        amount, token, amount_actual
    )
}

pub fn borrow_error(token: &AccountId, amount: u128, amount_actual: u128) -> String {
    format!(
        "You want to borrow {} of {} but only {} is available in reserve",
        amount, token, amount_actual
    )
}
