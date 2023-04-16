use crate::{
    msg::InstantiateMsg,
    state::{COUNTER, MINIMAL_DONATION, OWNER},
};
use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    COUNTER.save(deps.storage, &msg.counter)?;
    MINIMAL_DONATION.save(deps.storage, &msg.minimal_donation)?;
    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new())
}

pub mod query {
    use crate::{msg::ValueResponse, state::COUNTER};
    use cosmwasm_std::{Deps, StdResult};

    pub fn value(deps: Deps) -> StdResult<ValueResponse> {
        let value = COUNTER.load(deps.storage)?;
        Ok(ValueResponse { value })
    }
}

pub mod exec {
    use crate::{
        error::ContractError,
        state::{COUNTER, MINIMAL_DONATION, OWNER},
    };
    use cosmwasm_std::{BankMsg, DepsMut, Env, MessageInfo, Response, StdResult};

    pub fn reset(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;

        if owner != info.sender {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        COUNTER.save(deps.storage, &0)?;
        Ok(Response::new()
            .add_attribute("action", "reset")
            .add_attribute("sender", info.sender.as_str()))
    }

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;
        let mut value = COUNTER.load(deps.storage)?;
        if info.funds.iter().any(|coin| {
            coin.denom == minimal_donation.denom && coin.amount >= minimal_donation.amount
        }) {
            value += 1;
            COUNTER.save(deps.storage, &value)?;
        }
        // COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

        let resp = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", value.to_string());
        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        let funds = deps.querier.query_all_balances(&env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: funds,
        };
        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }
}
