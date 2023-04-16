use crate::{
    msg::InstantiateMsg,
    state::{State, OWNER, STATE},
};
use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_storage_plus::Item;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(
        deps.storage,
        &State {
            counter: msg.counter,
            minimal_donation: msg.minimal_donation,
        },
    )?;
    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new())
}

pub fn migrate(deps: DepsMut) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");

    let state = State {
        counter: COUNTER.load(deps.storage)?,
        minimal_donation: MINIMAL_DONATION.load(deps.storage)?,
    };

    STATE.save(deps.storage, &state)?;
    Ok(Response::new())
}

pub mod query {
    use crate::{msg::ValueResponse, state::STATE};
    use cosmwasm_std::{Deps, StdResult};

    pub fn value(deps: Deps) -> StdResult<ValueResponse> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValueResponse { value })
    }
}

pub mod exec {
    use crate::{
        error::ContractError,
        state::{State, OWNER, STATE},
    };
    use cosmwasm_std::{BankMsg, DepsMut, Env, MessageInfo, Response, StdResult};

    pub fn reset(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;

        if owner != info.sender {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }
        let State {
            counter: _,
            minimal_donation,
        } = STATE.load(deps.storage)?;

        STATE.save(
            deps.storage,
            &State {
                counter: 0,
                minimal_donation,
            },
        )?;
        Ok(Response::new()
            .add_attribute("action", "reset")
            .add_attribute("sender", info.sender.as_str()))
    }

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let mut state = STATE.load(deps.storage)?;
        if info.funds.iter().any(|coin| {
            coin.denom == state.minimal_donation.denom
                && coin.amount >= state.minimal_donation.amount
        }) {
            state.counter += 1;
            STATE.save(deps.storage, &state)?;
        }
        // COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

        let resp = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", state.counter.to_string());
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
