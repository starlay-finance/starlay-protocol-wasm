#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[cfg(test)]
mod tests;

/// Definition of WETH Gateway Contract
#[openbrush::contract]
pub mod weth_gateway {
    use ink::codegen::{
        EmitEvent,
        Env,
    };

    use logics::impls::weth_gateway::{
        Data,
        Internal,
        *,
    };
    use openbrush::{
        contracts::ownable::*,
        traits::Storage,
    };

    /// Contract's Storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct WETHGatewayContract {
        #[storage_field]
        gateway: Data,
        #[storage_field]
        ownable: ownable::Data,
    }

    /// Event: DepositETH is executed.
    #[ink(event)]
    pub struct DepositEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub from: AccountId,
        pub value: Balance,
    }

    /// Event: WithdrawEth is executed.
    #[ink(event)]
    pub struct WithdrawEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub to: AccountId,
        pub value: Balance,
    }

    /// Event: BorrowEth is executed.
    #[ink(event)]
    pub struct BorrowEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub to: AccountId,
        pub value: Balance,
    }

    /// Event: RepayEth is executed.
    #[ink(event)]
    pub struct RepayEth {
        #[ink(topic)]
        pub pool: AccountId,
        #[ink(topic)]
        pub from: AccountId,
        pub value: Balance,
    }

    impl Ownable for WETHGatewayContract {}

    impl Internal for WETHGatewayContract {
        fn _emit_deposit_eth_event_(&self, pool: AccountId, from: AccountId, value: Balance) {
            self.env()
                .emit_event::<DepositEth>(DepositEth { pool, from, value });
        }

        fn _emit_withdraw_eth_event_(&self, pool: AccountId, to: AccountId, value: Balance) {
            self.env()
                .emit_event::<WithdrawEth>(WithdrawEth { pool, to, value });
        }

        fn _emit_borrow_eth_event_(&self, pool: AccountId, to: AccountId, value: Balance) {
            self.env()
                .emit_event::<BorrowEth>(BorrowEth { pool, to, value });
        }

        fn _emit_repay_eth_event_(&self, pool: AccountId, from: AccountId, value: Balance) {
            self.env()
                .emit_event::<RepayEth>(RepayEth { pool, from, value });
        }
    }
    impl WETHGateway for WETHGatewayContract {}

    impl WETHGatewayContract {
        /// Generate this contract
        #[ink(constructor)]
        pub fn new(weth: AccountId) -> Self {
            let mut instance = Self::default();
            let caller = Self::env().caller();
            instance._init_with_owner(caller);
            instance.gateway.weth = weth;

            instance
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use controller::ControllerContractRef;
        use ink_e2e::{
            build_message,
            subxt::ext::sp_runtime::AccountId32,
            AccountKeyring,
        };
        use openbrush::traits::ZERO_ADDRESS;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn test_flipper(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let weth_gateway_constructor = WETHGatewayContractRef::new(ZERO_ADDRESS.into());
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

            Ok(())
        }
    }
}
