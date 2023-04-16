#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use error::ContractError;
use msg::{ExecMsg, InstantiateMsg};
mod contract;
pub mod error;
pub mod msg;
mod state;

#[cfg(any(test, feature = "tests"))]
pub mod multitest;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    contract::migrate(deps)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use contract::exec::*;
    use ExecMsg::*;

    match msg {
        Donate {} => donate(deps, info).map_err(ContractError::from),
        Reset { counter: _ } => reset(deps, info),
        Withdraw {} => withdraw(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    match msg {
        Value {} => to_binary(&contract::query::value(deps)?),
        ValueIncremented { value } => {
            let resp = msg::ValueResponse { value: value + 1 };
            to_binary(&resp)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use cosmwasm_std::{coins, Addr, Coin, Empty};
//     use cw_multi_test::{App, Contract, ContractWrapper, Executor};
//     const ATOM: &str = "atom";
//     use crate::{
//         error::ContractError,
//         execute, instantiate,
//         msg::{ExecMsg, InstantiateMsg, QueryMsg::*, ValueResponse},
//         query,
//     };

//     fn counting_contract() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(execute, instantiate, query);
//         Box::new(contract)
//     }

//     #[test]
//     fn query_value() {
//         let mut app = App::default();
//         let contract_id = app.store_code(counting_contract());
//         let contract_addr = app
//             .instantiate_contract(
//                 contract_id,
//                 Addr::unchecked("sender"),
//                 &InstantiateMsg {
//                     counter: 0,
//                     minimal_donation: Coin::new(10, ATOM),
//                 },
//                 &[],
//                 "Counting Contract",
//                 None,
//             )
//             .unwrap();

//         let resp: ValueResponse = app
//             .wrap()
//             .query_wasm_smart(contract_addr, &Value {})
//             .unwrap();

//         assert_eq!(resp, ValueResponse { value: 0 });
//     }

//     #[test]
//     fn donate() {
//         let mut app = App::default();
//         let sender = Addr::unchecked("sender");

//         let contract_id = app.store_code(counting_contract());
//         let contract_addr = app
//             .instantiate_contract(
//                 contract_id,
//                 sender.clone(),
//                 &InstantiateMsg {
//                     counter: 0,
//                     minimal_donation: Coin::new(10, ATOM),
//                 },
//                 &[],
//                 "Counting Contract",
//                 None,
//             )
//             .unwrap();

//         app.execute_contract(sender, contract_addr.clone(), &ExecMsg::Donate {}, &[])
//             .unwrap();

//         let resp: ValueResponse = app
//             .wrap()
//             .query_wasm_smart(contract_addr, &Value {})
//             .unwrap();

//         assert_eq!(resp, ValueResponse { value: 0 });
//     }

//     #[test]
//     fn donate_with_funds() {
//         let sender = Addr::unchecked("sender");

//         let mut app = App::new(|router, _api, storage| {
//             router
//                 .bank
//                 .init_balance(storage, &sender, coins(10, ATOM))
//                 .unwrap();
//         });
//         let contract_id = app.store_code(counting_contract());
//         let contract_addr = app
//             .instantiate_contract(
//                 contract_id,
//                 sender.clone(),
//                 &InstantiateMsg {
//                     counter: 0,
//                     minimal_donation: Coin::new(10, ATOM),
//                 },
//                 &[],
//                 "Counting Contract",
//                 None,
//             )
//             .unwrap();

//         app.execute_contract(
//             sender.clone(),
//             contract_addr.clone(),
//             &ExecMsg::Donate {},
//             &coins(10, ATOM),
//         )
//         .unwrap();

//         let resp: ValueResponse = app
//             .wrap()
//             .query_wasm_smart(contract_addr.clone(), &Value {})
//             .unwrap();

//         assert_eq!(resp, ValueResponse { value: 1 });

//         assert_eq!(app.wrap().query_all_balances(sender).unwrap(), []);
//         assert_eq!(
//             app.wrap().query_all_balances(contract_addr).unwrap(),
//             coins(10, ATOM)
//         );
//     }

//     #[test]
//     fn withdraw() {
//         let owner = Addr::unchecked("owner");
//         let sender1 = Addr::unchecked("sender1");
//         let sender2 = Addr::unchecked("sender2");

//         let mut app = App::new(|router, _api, storage| {
//             router
//                 .bank
//                 .init_balance(storage, &sender1, coins(10, ATOM))
//                 .unwrap();

//             router
//                 .bank
//                 .init_balance(storage, &sender2, coins(5, ATOM))
//                 .unwrap();
//         });
//         let contract_id = app.store_code(counting_contract());
//         let contract_addr = app
//             .instantiate_contract(
//                 contract_id,
//                 owner.clone(),
//                 &InstantiateMsg {
//                     counter: 0,
//                     minimal_donation: Coin::new(10, ATOM),
//                 },
//                 &[],
//                 "Counting Contract",
//                 None,
//             )
//             .unwrap();

//         app.execute_contract(
//             sender1.clone(),
//             contract_addr.clone(),
//             &ExecMsg::Donate {},
//             &coins(10, ATOM),
//         )
//         .unwrap();

//         app.execute_contract(
//             sender2.clone(),
//             contract_addr.clone(),
//             &ExecMsg::Donate {},
//             &coins(5, ATOM),
//         )
//         .unwrap();

//         app.execute_contract(
//             owner.clone(),
//             contract_addr.clone(),
//             &ExecMsg::Withdraw {},
//             &[],
//         )
//         .unwrap();

//         assert_eq!(
//             app.wrap().query_all_balances(owner).unwrap(),
//             coins(15, ATOM)
//         );
//         assert_eq!(
//             app.wrap().query_all_balances(contract_addr).unwrap(),
//             vec![],
//         );
//         assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), vec![],);
//         assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), vec![],);
//     }

//     #[test]
//     fn unauthorized_withdraw() {
//         let owner = Addr::unchecked("owner");
//         let member = Addr::unchecked("member");

//         let mut app = App::default();
//         let contract_id = app.store_code(counting_contract());
//         let contract_addr = app
//             .instantiate_contract(
//                 contract_id,
//                 owner.clone(),
//                 &InstantiateMsg {
//                     counter: 0,
//                     minimal_donation: Coin::new(10, ATOM),
//                 },
//                 &[],
//                 "Counting Contract",
//                 None,
//             )
//             .unwrap();

//         let err = app
//             .execute_contract(member, contract_addr, &ExecMsg::Withdraw {}, &[])
//             .unwrap_err();

//         assert_eq!(
//             ContractError::Unauthorized {
//                 owner: owner.to_string()
//             },
//             err.downcast().unwrap()
//         );
//     }
// }
