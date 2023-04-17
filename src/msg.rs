use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};

#[cw_serde]
pub struct Parent {
    pub addr: String,
    pub donating_period: u64,
    pub part: Decimal,
}

#[cw_serde]
pub struct InstantiateMsg {
    #[serde(default)]
    pub counter: u64,
    pub minimal_donation: Coin,
    pub parent: Option<Parent>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ValueResponse)]
    Value {},
    #[returns(ValueResponse)]
    ValueIncremented { value: u64 },
}

#[cw_serde]
pub enum ExecMsg {
    Donate {},
    Reset {
        #[serde(default)]
        counter: u64,
    },
    Withdraw {},
}

#[cw_serde]
pub struct ValueResponse {
    pub value: u64,
}
