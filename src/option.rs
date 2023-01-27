enum OptionType {
    Put,
    Call,
}

pub struct Option {
    pub price: f64,
    pub expiration_ts: u64,
    pub option_type: OptionType,
    pub insurance: u128,
    pub token: AccountId,
}