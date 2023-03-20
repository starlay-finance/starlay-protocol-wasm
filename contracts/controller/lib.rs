#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod contract {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use logics::impls::controller::{
        Internal,
        *,
    };
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct ControllerContract {
        #[storage_field]
        controller: Data,
    }

    #[ink(event)]
    pub struct MarketListed {
        pool: AccountId,
    }

    impl Controller for ControllerContract {}

    impl ControllerContract {
        #[ink(constructor)]
        pub fn new(manager: AccountId) -> Self {
            let mut instance = Self::default();
            instance.controller.manager = manager;
            instance
        }
    }

    impl Internal for ControllerContract {
        fn _emit_market_listed_event(&self, pool: AccountId) {
            self.env().emit_event(MarketListed { pool });
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::ops::{
            Add,
            Div,
            Mul,
        };
        use ink::env::{
            test::{
                self,
                recorded_events,
                DefaultAccounts,
                EmittedEvent,
            },
            DefaultEnvironment,
        };
        use logics::{
            impls::exp_no_err::exp_scale,
            traits::types::WrappedU256,
        };
        use openbrush::traits::ZERO_ADDRESS;
        use primitive_types::U256;

        type Event = <ControllerContract as ink::reflect::ContractEventBase>::Type;

        fn default_accounts() -> DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }
        fn set_caller(id: AccountId) {
            test::set_caller::<DefaultEnvironment>(id);
        }
        fn get_emitted_events() -> Vec<EmittedEvent> {
            recorded_events().collect::<Vec<_>>()
        }
        fn decode_market_listed_event(event: EmittedEvent) -> MarketListed {
            if let Ok(Event::MarketListed(x)) =
                <Event as scale::Decode>::decode(&mut &event.data[..])
            {
                return x
            }
            panic!("unexpected event kind: expected MarketListed event")
        }

        #[ink::test]
        fn new_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let contract = ControllerContract::new(accounts.bob);
            assert_eq!(contract.markets(), []);
            assert_eq!(contract.oracle(), ZERO_ADDRESS.into());
            assert_eq!(contract.manager(), accounts.bob);
            assert_eq!(contract.close_factor_mantissa(), WrappedU256::from(0));
            assert_eq!(
                contract.liquidation_incentive_mantissa(),
                WrappedU256::from(0)
            );
        }

        #[ink::test]
        fn mint_allowed_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert!(contract.support_market(pool).is_ok());
            assert!(contract.mint_allowed(pool, accounts.bob, 0).is_ok());
        }

        #[ink::test]
        fn mint_allowed_fail_when_not_supported() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert_eq!(
                contract.mint_allowed(pool, accounts.bob, 0).unwrap_err(),
                Error::MintIsPaused
            );
        }

        #[ink::test]
        fn mint_allowed_fail_when_paused() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert!(contract.support_market(pool).is_ok());
            assert!(contract.set_mint_guardian_paused(pool, true).is_ok());
            assert_eq!(
                contract.mint_allowed(pool, accounts.bob, 0).unwrap_err(),
                Error::MintIsPaused
            );
        }

        #[ink::test]
        fn borrow_allowed_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert!(contract.support_market(pool).is_ok());
            assert!(contract.borrow_allowed(pool, accounts.bob, 0).is_ok());
        }

        #[ink::test]
        fn borrow_allowed_fail_when_not_supported() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert_eq!(
                contract.borrow_allowed(pool, accounts.bob, 0).unwrap_err(),
                Error::BorrowIsPaused
            );
        }

        #[ink::test]
        fn borrow_allowed_fail_when_paused() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert!(contract.support_market(pool).is_ok());
            assert!(contract.set_borrow_guardian_paused(pool, true).is_ok());
            assert_eq!(
                contract.borrow_allowed(pool, accounts.bob, 0).unwrap_err(),
                Error::BorrowIsPaused
            );
        }

        // TODO
        // #[ink::test]
        // fn liquidate_borrow_allowed_works() {
        //     let accounts = default_accounts();
        //     set_caller(accounts.bob);
        //     let mut contract = ControllerContract::new(accounts.bob);

        //     let pool1 = AccountId::from([0x01; 32]);
        //     let pool2 = AccountId::from([0x02; 32]);
        //     assert!(contract.support_market(pool1).is_ok());
        //     assert!(contract.support_market(pool2).is_ok());
        //     assert!(contract
        //         .liquidate_borrow_allowed(pool1, pool2, ZERO_ADDRESS.into(), ZERO_ADDRESS.into(), 0)
        //         .is_ok())
        // }

        #[ink::test]
        fn liquidate_borrow_allowed_fail() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            // not in market
            let pool1 = AccountId::from([0x01; 32]);
            let pool2 = AccountId::from([0x02; 32]);
            assert_eq!(
                contract
                    .liquidate_borrow_allowed(
                        pool1,
                        pool2,
                        ZERO_ADDRESS.into(),
                        ZERO_ADDRESS.into(),
                        0
                    )
                    .unwrap_err(),
                Error::MarketNotListed
            );
            assert!(contract.support_market(pool1).is_ok());
            assert_eq!(
                contract
                    .liquidate_borrow_allowed(
                        pool1,
                        pool2,
                        ZERO_ADDRESS.into(),
                        ZERO_ADDRESS.into(),
                        0
                    )
                    .unwrap_err(),
                Error::MarketNotListed
            );
        }

        #[ink::test]
        fn seize_allowed_fail() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            // not in market
            let pool1 = AccountId::from([0x01; 32]);
            let pool2 = AccountId::from([0x02; 32]);
            assert_eq!(
                contract
                    .seize_allowed(pool1, pool2, ZERO_ADDRESS.into(), ZERO_ADDRESS.into(), 0)
                    .unwrap_err(),
                Error::MarketNotListed
            );
            assert!(contract.support_market(pool1).is_ok());
            assert_eq!(
                contract
                    .seize_allowed(pool1, pool2, ZERO_ADDRESS.into(), ZERO_ADDRESS.into(), 0)
                    .unwrap_err(),
                Error::MarketNotListed
            );
        }

        #[ink::test]
        fn support_market_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            let p1 = AccountId::from([0x01; 32]);
            assert!(contract.support_market(p1).is_ok());
            assert_eq!(contract.markets(), [p1]);
            assert_eq!(
                contract.collateral_factor_mantissa(p1),
                Some(WrappedU256::from(0))
            );
            assert_eq!(contract.mint_guardian_paused(p1), Some(false));
            assert_eq!(contract.borrow_guardian_paused(p1), Some(false));
            assert_eq!(contract.borrow_cap(p1), Some(0));
            let event = decode_market_listed_event(get_emitted_events()[0].clone());
            assert_eq!(event.pool, p1);

            let p2 = AccountId::from([0x02; 32]);
            assert!(contract.support_market(p2).is_ok());
            assert_eq!(contract.markets(), [p1, p2]);
        }

        #[ink::test]
        fn set_collateral_factor_mantissa_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);
            let pool_addr = AccountId::from([0x01; 32]);
            assert_eq!(contract.collateral_factor_mantissa(pool_addr), None);

            let max = exp_scale().mul(U256::from(90)).div(U256::from(100));
            assert!(contract
                .set_collateral_factor_mantissa(pool_addr, WrappedU256::from(max))
                .is_ok());
            assert_eq!(
                contract.collateral_factor_mantissa(pool_addr),
                Some(WrappedU256::from(max))
            );

            assert_eq!(
                contract
                    .set_collateral_factor_mantissa(pool_addr, WrappedU256::from(max.add(1)))
                    .unwrap_err(),
                Error::InvalidCollateralFactor
            );
        }

        #[ink::test]
        fn mint_guardian_paused_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert_eq!(contract.mint_guardian_paused(pool), None);

            assert!(contract.support_market(pool).is_ok());
            assert_eq!(contract.mint_guardian_paused(pool), Some(false));

            assert!(contract.set_mint_guardian_paused(pool, true).is_ok());
            assert_eq!(contract.mint_guardian_paused(pool), Some(true));
            assert!(contract.set_mint_guardian_paused(pool, false).is_ok());
            assert_eq!(contract.mint_guardian_paused(pool), Some(false));
        }

        #[ink::test]
        fn borrow_guardian_paused_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new(accounts.bob);

            let pool = AccountId::from([0x01; 32]);
            assert_eq!(contract.borrow_guardian_paused(pool), None);

            assert!(contract.support_market(pool).is_ok());
            assert_eq!(contract.mint_guardian_paused(pool), Some(false));

            assert!(contract.set_borrow_guardian_paused(pool, true).is_ok());
            assert_eq!(contract.borrow_guardian_paused(pool), Some(true));
            assert!(contract.set_borrow_guardian_paused(pool, false).is_ok());
            assert_eq!(contract.borrow_guardian_paused(pool), Some(false));
        }

        #[ink::test]
        fn assert_manager_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);

            let mut contract = ControllerContract::new(accounts.bob);

            set_caller(accounts.charlie);
            let dummy_id = AccountId::from([0xff; 32]);
            assert_eq!(
                contract.set_price_oracle(dummy_id).unwrap_err(),
                Error::CallerIsNotManager
            );
            assert_eq!(
                contract.support_market(dummy_id).unwrap_err(),
                Error::CallerIsNotManager
            );
            assert_eq!(
                contract
                    .set_collateral_factor_mantissa(dummy_id, WrappedU256::from(0))
                    .unwrap_err(),
                Error::CallerIsNotManager
            );
            assert_eq!(
                contract
                    .set_mint_guardian_paused(dummy_id, true)
                    .unwrap_err(),
                Error::CallerIsNotManager
            );
            assert_eq!(
                contract
                    .set_borrow_guardian_paused(dummy_id, true)
                    .unwrap_err(),
                Error::CallerIsNotManager
            );
            assert_eq!(
                contract
                    .set_close_factor_mantissa(WrappedU256::from(0))
                    .unwrap_err(),
                Error::CallerIsNotManager
            );
            assert_eq!(
                contract
                    .set_liquidation_incentive_mantissa(WrappedU256::from(0))
                    .unwrap_err(),
                Error::CallerIsNotManager
            );
            assert_eq!(
                contract.set_borrow_cap(dummy_id, 0).unwrap_err(),
                Error::CallerIsNotManager
            );
        }
    }
}
