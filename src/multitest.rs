use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    execute, instantiate,
    msg::{ExecMsg, InstantiateMsg, QueryMsg, ValueResponse},
    query,
};

#[cfg(test)]
mod tests;

pub struct CountingContract(Addr);

impl CountingContract {
    #[track_caller]
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        admin: impl Into<Option<&'a Addr>>,
        counter: impl Into<Option<u64>>,
        minimal_donation: Coin,
    ) -> StdResult<CountingContract> {
        let counter = counter.into().unwrap_or_default();
        let admin = admin.into();
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                minimal_donation,
                counter,
            },
            &[],
            label,
            admin.map(Addr::to_string),
        )
        .map_err(|err| err.downcast().unwrap())
        .map(CountingContract)
    }

    #[track_caller]
    pub fn donate(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Donate {}, funds)
            .map_err(|err| err.downcast::<ContractError>().unwrap())?;
        Ok(())
    }

    #[track_caller]
    pub fn withdraw(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Withdraw {}, &[])
            .map_err(|err| err.downcast::<ContractError>().unwrap())?;
        Ok(())
    }

    #[track_caller]
    pub fn query_value(&self, app: &App) -> StdResult<ValueResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Value {})
    }
}

impl From<CountingContract> for Addr {
    fn from(contract: CountingContract) -> Self {
        contract.0
    }
}
