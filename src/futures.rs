enum FuturesType {
    Deliverable,
    Settlement,
}

pub struct Futures {
    pub price: f64,
    pub expiration_ts: u64,
    pub futures_type: FuturesType,
    pub token: AccountId,
    pub collateral: u128,
}