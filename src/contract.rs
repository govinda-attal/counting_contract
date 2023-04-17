use crate::error::ContractError;
use crate::state::{ParentDonation, PARENT_DONATION};
use crate::{
    msg::InstantiateMsg,
    state::{State, STATE},
};
use cosmwasm_std::{Addr, Coin, DepsMut, MessageInfo, Response, StdResult};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(
        deps.storage,
        &State {
            counter: msg.counter,
            minimal_donation: msg.minimal_donation,
            owner: info.sender,
            donating_parent: msg.parent.as_ref().map(|p| p.donating_period),
        },
    )?;

    if let Some(parent) = msg.parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            },
        )?;
    }

    Ok(Response::new())
}

pub fn migrate(mut deps: DepsMut) -> Result<Response, ContractError> {
    let ContractVersion { contract, version } = get_contract_version(deps.storage)?;
    if contract != CONTRACT_NAME {
        return Err(ContractError::InvalidName(contract));
    }
    let resp = match version.as_str() {
        "0.1.4" => migrate_0_1(deps.branch()).map_err(ContractError::from)?,
        "0.2.0" => migrate_0_2(deps.branch()).map_err(ContractError::from)?,
        CONTRACT_VERSION => return Ok(Response::new()),
        _ => return Err(ContractError::UnsupportedVersion(version)),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(resp)
}

pub fn migrate_0_1(deps: DepsMut) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");
    const OWNER: Item<Addr> = Item::new("owner");

    let counter = COUNTER.load(deps.storage)?;
    let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;

    STATE.save(
        deps.storage,
        &State {
            counter,
            minimal_donation,
            owner,
            donating_parent: None,
        },
    )?;

    Ok(Response::new())
}

pub fn migrate_0_2(deps: DepsMut) -> StdResult<Response> {
    #[derive(Serialize, Deserialize)]
    struct OldState {
        counter: u64,
        minimal_donation: Coin,
        owner: Addr,
    }

    const OLD_STATE: Item<OldState> = Item::new("state");

    let OldState {
        counter,
        minimal_donation,
        owner,
    } = OLD_STATE.load(deps.storage)?;

    STATE.save(
        deps.storage,
        &State {
            counter,
            minimal_donation,
            owner,
            donating_parent: None,
        },
    )?;

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
        msg::ExecMsg,
        state::{State, PARENT_DONATION, STATE},
    };
    use cosmwasm_std::{
        to_binary, BankMsg, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
    };

    pub fn reset(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let State {
            counter: _,
            minimal_donation,
            donating_parent,
            owner,
        } = STATE.load(deps.storage)?;

        if owner != info.sender {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        STATE.save(
            deps.storage,
            &State {
                counter: 0,
                minimal_donation,
                donating_parent,
                owner,
            },
        )?;
        Ok(Response::new()
            .add_attribute("action", "reset")
            .add_attribute("sender", info.sender.as_str()))
    }

    pub fn donate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        let mut state = STATE.load(deps.storage)?;
        let mut resp = Response::new();
        if state.minimal_donation.amount.is_zero()
            || info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom
                    && coin.amount >= state.minimal_donation.amount
            })
        {
            state.counter += 1;
            if let Some(parent) = &mut state.donating_parent {
                *parent -= 1;
                if *parent == 0 {
                    let parent_donation = PARENT_DONATION.load(deps.storage)?;
                    *parent = parent_donation.donating_parent_period;

                    let funds: Vec<_> = deps
                        .querier
                        .query_all_balances(env.contract.address)?
                        .into_iter()
                        .map(|mut coin| {
                            coin.amount = coin.amount * parent_donation.part;
                            coin
                        })
                        .collect();

                    let msg = WasmMsg::Execute {
                        contract_addr: parent_donation.address.to_string(),
                        msg: to_binary(&ExecMsg::Donate {})?,
                        funds,
                    };
                    resp = resp
                        .add_message(msg)
                        .add_attribute("donated_to_parent", parent_donation.address.to_string());
                }
            }
            STATE.save(deps.storage, &state)?;
        }
        // COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

        resp = resp
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", state.counter.to_string());
        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let State {
            counter: _,
            minimal_donation: _,
            donating_parent: _,
            owner,
        } = STATE.load(deps.storage)?;

        if owner != info.sender {
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
