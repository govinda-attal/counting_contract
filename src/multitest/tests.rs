use crate::{
    error::ContractError,
    execute, instantiate,
    msg::ValueResponse,
    multitest::CountingContract,
    query,
    state::{State, STATE},
};
use cosmwasm_std::{coin, coins, Addr, Coin, Empty};
use counting_contract_0_1_3::multitest::CountingContract as CountingContract_0_1;
use cw_multi_test::{App, Contract, ContractWrapper};

fn counting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

const ATOM: &str = "atom";

#[test]
fn query_value() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let contract_id = app.store_code(counting_contract());
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        None,
        0,
        Coin::new(10, ATOM),
    )
    .unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResponse { value: 0 });
}

#[test]
fn donate() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let contract_id = app.store_code(counting_contract());
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        None,
        0,
        Coin::new(10, ATOM),
    )
    .unwrap();

    contract.donate(&mut app, &sender, &[]).unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResponse { value: 0 });
}

#[test]
fn donate_with_funds() {
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });
    let contract_id = app.store_code(counting_contract());
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        None,
        0,
        Coin::new(10, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResponse { value: 1 });

    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), []);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(10, ATOM)
    );
}

#[test]
fn withdraw() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &sender2, coins(5, ATOM))
            .unwrap();
    });
    let contract_id = app.store_code(counting_contract());
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        "Counting contract",
        None,
        0,
        Coin::new(10, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender1, &coins(10, ATOM))
        .unwrap();

    contract
        .donate(&mut app, &sender2, &coins(5, ATOM))
        .unwrap();

    contract.withdraw(&mut app, &owner).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(15, ATOM)
    );
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        vec![],
    );
    assert_eq!(app.wrap().query_all_balances(sender1).unwrap(), vec![],);
    assert_eq!(app.wrap().query_all_balances(sender2).unwrap(), vec![],);
}

#[test]
fn unauthorized_withdraw() {
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");

    let mut app = App::default();
    let contract_id = app.store_code(counting_contract());
    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        "Counting contract",
        None,
        0,
        Coin::new(10, ATOM),
    )
    .unwrap();

    let err = contract.withdraw(&mut app, &member).unwrap_err();

    assert_eq!(
        err,
        ContractError::Unauthorized {
            owner: owner.into()
        },
    );
}

#[test]
fn migrate() {
    let admin = Addr::unchecked("admin");
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });

    let old_code_id = CountingContract_0_1::store_code(&mut app);
    let new_code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract_0_1::instantiate(
        &mut app,
        old_code_id,
        &owner,
        "Counting contract",
        Some(&admin),
        None,
        coin(10, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let contract =
        CountingContract::migrate(&mut app, contract.into(), new_code_id, &admin).unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResponse { value: 1 });

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            counter: 1,
            minimal_donation: coin(10, ATOM)
        }
    );
}


#[test]
fn migrate_no_update() {
    let admin = Addr::unchecked("admin");
    let owner = Addr::unchecked("owner");
    let mut app = App::default();

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        &admin,
        None,
        coin(10, ATOM),
    )
    .unwrap();

    CountingContract::migrate(&mut app, contract.into(), code_id, &admin).unwrap();
}
