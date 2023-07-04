use crate::weth_gateway::WETHGatewayContractRef;
use controller::controller::ControllerContractRef;
use default_interest_rate_model::default_interest_rate_model::DefaultInterestRateModelContractRef;
use pool::pool::PoolContractRef;
use price_oracle::price_oracle::PriceOracleContractRef;
use weth::weth::WETHContractRef;

use logics::traits::{
    controller::Controller,
    types::WrappedU256,
};

use core::ops::{
    Add,
    Div,
    Mul,
    Sub,
};
use primitive_types::U256;
use serial_test::serial;

#[cfg(all(test, feature = "e2e-tests"))]
use ink_e2e::{
    build_message,
    subxt::ext::sp_runtime::AccountId32,
    AccountKeyring,
};
use openbrush::{
    contracts::psp22::extensions::metadata::PSP22Metadata,
    traits::{
        AccountId,
        ZERO_ADDRESS,
    },
};
type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Global Variable
static mut CONTROLLER_ID: Option<AccountId> = None;
static mut WETH_GATEWAY_ID: Option<AccountId> = None;
static mut PRICE_ORACLE_ID: Option<AccountId> = None;
static mut RATE_MODEL_ID: Option<AccountId> = None;
static mut WETH_ID: Option<AccountId> = None;
static mut POOL_ID: Option<AccountId> = None;

#[ink_e2e::test]
#[serial]
async fn _1_initialize_e2e_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    // Deploy Controller
    let controller_constructor =
        ControllerContractRef::new(ink_e2e::account_id(AccountKeyring::Alice));
    let controller_id = client
        .instantiate(
            "controller",
            &ink_e2e::alice(),
            controller_constructor,
            0,
            None,
        )
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(controller_id != AccountId::from([0x0; 32]));

    // Deploy Price Oracle
    let price_oracle_constructor = PriceOracleContractRef::new();
    let price_oracle_id = client
        .instantiate(
            "price_oracle",
            &ink_e2e::alice(),
            price_oracle_constructor,
            0,
            None,
        )
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(price_oracle_id != AccountId::from([0x0; 32]));

    // Deploy Default Interest Model
    let one_ether: U256 = U256::from(10_u128.pow(18));
    let rate_model_arg: WrappedU256 = WrappedU256::from(U256::from(100).mul(one_ether));

    let rate_model_constructor = DefaultInterestRateModelContractRef::new(
        rate_model_arg,
        rate_model_arg,
        rate_model_arg,
        rate_model_arg,
    );
    let rate_model_id = client
        .instantiate(
            "default_interest_rate_model",
            &ink_e2e::alice(),
            rate_model_constructor,
            0,
            None,
        )
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(rate_model_id != AccountId::from([0x0; 32]));

    // Deploy WETH
    let weth_constructor = WETHContractRef::new();
    let weth_id = client
        .instantiate("weth", &ink_e2e::alice(), weth_constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(weth_id != AccountId::from([0x0; 32]));

    // Deploy WETH Gateway
    let weth_gateway_constructor = WETHGatewayContractRef::new(weth_id);
    let weth_gateway_id = client
        .instantiate(
            "weth_gateway",
            &ink_e2e::alice(),
            weth_gateway_constructor,
            0,
            None,
        )
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(weth_gateway_id != AccountId::from([0x0; 32]));

    // Prepare Pool with Token
    // let get_token_name =
    //     build_message::<WETHContractRef>(weth_id.clone()).call(|weth| weth.token_name());
    // let get_token_name_result = client
    //     .call_dry_run(&ink_e2e::bob(), &get_token_name, 0, None)
    //     .await
    //     .return_value();

    // let get_token_symbol =
    //     build_message::<WETHContractRef>(weth_id.clone()).call(|weth| weth.token_symbol());
    // let get_token_symbol_result = client
    //     .call_dry_run(&ink_e2e::bob(), &get_token_symbol, 0, None)
    //     .await
    //     .return_value();

    // let get_token_decimals =
    //     build_message::<WETHContractRef>(weth_id.clone()).call(|weth| weth.token_decimals());
    // let get_token_decimals_result = client
    //     .call_dry_run(&ink_e2e::bob(), &get_token_decimals, 0, None)
    //     .await
    //     .return_value();

    // assert_eq!(get_token_decimals_result, 18);

    // Set Global Variable for AccountIds
    unsafe {
        CONTROLLER_ID = Some(controller_id);
        WETH_GATEWAY_ID = Some(weth_gateway_id);
        PRICE_ORACLE_ID = Some(price_oracle_id);
        RATE_MODEL_ID = Some(rate_model_id);
        WETH_ID = Some(weth_id);
    }

    Ok(())
}

#[ink_e2e::test]
#[serial]
async fn _2_deposit_eth(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    let controller_id: AccountId;
    unsafe {
        assert!(!CONTROLLER_ID.is_none());
        controller_id = CONTROLLER_ID.unwrap();
    }
    let get_manager = build_message::<ControllerContractRef>(controller_id.clone())
        .call(|controller| controller.manager());
    let manager = client
        .call_dry_run(&ink_e2e::alice(), &get_manager, 0, None)
        .await
        .return_value();

    assert_eq!(ink_e2e::account_id(AccountKeyring::Alice), manager);

    Ok(())
}
