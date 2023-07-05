use crate::weth_gateway::WETHGatewayContractRef;
use controller::controller::ControllerContractRef;
use default_interest_rate_model::default_interest_rate_model::DefaultInterestRateModelContractRef;
use pool::pool::PoolContractRef;
use price_oracle::price_oracle::PriceOracleContractRef;
use weth::weth::WETHContractRef;

use logics::{
    impls::{
        controller::controller_external::Controller,
        price_oracle::priceoracle_external::PriceOracle,
        weth_gateway::wethgateway_external::WETHGateway,
    },
    traits::types::WrappedU256,
};

use core::ops::Mul;
use primitive_types::U256;

#[cfg(all(test, feature = "e2e-tests"))]
use ink_e2e::{
    account_id,
    alice,
    build_message,
    AccountKeyring,
};
use openbrush::{
    contracts::psp22::extensions::metadata::psp22metadata_external::PSP22Metadata,
    traits::AccountId,
};
type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn _1_initialize_e2e_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    let one_ether = 10_u128.pow(18);
    let rate_model_arg: WrappedU256 = WrappedU256::from(U256::from(100).mul(U256::from(one_ether)));
    // Deploy Controller
    let controller_constructor = ControllerContractRef::new(account_id(AccountKeyring::Alice));
    let controller_id = client
        .instantiate("controller", &alice(), controller_constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(controller_id != AccountId::from([0x0; 32]));

    // Deploy Price Oracle
    let price_oracle_constructor = PriceOracleContractRef::new();
    let price_oracle_id = client
        .instantiate("price_oracle", &alice(), price_oracle_constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(price_oracle_id != AccountId::from([0x0; 32]));

    // Deploy Default Interest Model
    let rate_model_constructor = DefaultInterestRateModelContractRef::new(
        rate_model_arg,
        rate_model_arg,
        rate_model_arg,
        rate_model_arg,
    );
    let rate_model_id = client
        .instantiate(
            "default_interest_rate_model",
            &alice(),
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
        .instantiate("weth", &alice(), weth_constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(weth_id != AccountId::from([0x0; 32]));

    // Deploy WETH Gateway
    let weth_gateway_constructor = WETHGatewayContractRef::new(weth_id);
    let weth_gateway_id = client
        .instantiate("weth_gateway", &alice(), weth_gateway_constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(weth_gateway_id != AccountId::from([0x0; 32]));

    // Prepare Pool with Token
    let get_token_name = build_message::<WETHContractRef>(weth_id).call(|weth| weth.token_name());
    let get_token_name_result = client
        .call_dry_run(&alice(), &get_token_name, 0, None)
        .await
        .return_value();
    assert!(!get_token_name_result.is_none());
    let token_name = get_token_name_result.unwrap();
    assert_eq!(token_name, "Wrapped Astar");

    let get_token_symbol =
        build_message::<WETHContractRef>(weth_id).call(|weth| weth.token_symbol());
    let get_token_symbol_result = client
        .call_dry_run(&alice(), &get_token_symbol, 0, None)
        .await
        .return_value();
    assert!(!get_token_symbol_result.is_none());
    let token_symbol = get_token_symbol_result.unwrap();
    assert_eq!(token_symbol, "WASTR");

    let get_token_decimals =
        build_message::<WETHContractRef>(weth_id).call(|weth| weth.token_decimals());
    let token_decimals = client
        .call_dry_run(&alice(), &get_token_decimals, 0, None)
        .await
        .return_value();
    assert_eq!(token_decimals, 18);

    // Deploy Pool
    let pool_constructor = PoolContractRef::new(
        weth_id,
        controller_id,
        rate_model_id,
        WrappedU256::from(one_ether),
        10000,
        token_name,
        token_symbol,
        token_decimals,
    );
    let pool_id = client
        .instantiate("pool", &alice(), pool_constructor, 0, None)
        .await
        .expect("instantiate failed")
        .account_id;
    assert!(pool_id != AccountId::from([0x0; 32]));

    // Initialize Controller
    let set_price_oracle = build_message::<ControllerContractRef>(controller_id)
        .call(|controller| controller.set_price_oracle(price_oracle_id));
    client
        .call(&alice(), set_price_oracle, 0, None)
        .await
        .expect("Failed to set Price Oracle");

    let set_close_factor_mantissa = build_message::<ControllerContractRef>(controller_id)
        .call(|controller| controller.set_close_factor_mantissa(WrappedU256::from(one_ether)));
    client
        .call(&alice(), set_close_factor_mantissa, 0, None)
        .await
        .expect("Failed to set Close Factor Mantissa");

    // List Pool to controller
    let set_fixed_price_weth = build_message::<PriceOracleContractRef>(price_oracle_id)
        .call(|price_oracle| price_oracle.set_fixed_price(weth_id, one_ether));
    client
        .call(&alice(), set_fixed_price_weth, 0, None)
        .await
        .expect("Failed to set Price of WETH");

    let support_market = build_message::<ControllerContractRef>(controller_id).call(|controller| {
        controller.support_market_with_collateral_factor_mantissa(
            pool_id,
            weth_id,
            WrappedU256::from(U256::from(one_ether * 90 / 100)),
        )
    });
    client
        .call(&alice(), support_market, 0, None)
        .await
        .expect("Failed to Support Market");

    // instantiate test
    let get_weth_address = build_message::<WETHGatewayContractRef>(weth_gateway_id)
        .call(|weth_gateway| weth_gateway.get_weth_address());
    let get_weth_address_result = client
        .call_dry_run(&alice(), &get_weth_address, 0, None)
        .await
        .return_value();
    assert_eq!(get_weth_address_result, weth_id);

    // Deposit Test
    let before_deposit_weth_contract_balance = client
        .balance(weth_id)
        .await
        .expect("Failed to get balance");

    let deposit_amount: u128 = 3000;
    let deposit_eth = build_message::<WETHGatewayContractRef>(weth_gateway_id)
        .call(|weth_gateway| weth_gateway.deposit_eth(pool_id));
    client
        .call(&alice(), deposit_eth, deposit_amount, None)
        .await
        .expect("Failed to Deposit Eth");

    let after_deposit_weth_contract_balance = client
        .balance(weth_id)
        .await
        .expect("Failed to get balance");

    assert_eq!(
        before_deposit_weth_contract_balance + deposit_amount,
        after_deposit_weth_contract_balance
    );

    Ok(())
}
