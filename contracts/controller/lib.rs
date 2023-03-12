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
    #[derive(Storage)]
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
        pub fn new() -> Self {
            Self {
                controller: Data {
                    markets: Default::default(),
                },
            }
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
        use ink::env::{
            test::{
                self,
                recorded_events,
                DefaultAccounts,
                EmittedEvent,
            },
            DefaultEnvironment,
        };

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

            let contract = ControllerContract::new();
            assert_eq!(contract.markets(), []);
        }

        #[ink::test]
        fn support_market_works() {
            let accounts = default_accounts();
            set_caller(accounts.bob);
            let mut contract = ControllerContract::new();

            let p1 = AccountId::from([0x01; 32]);
            assert!(contract.support_market(p1).is_ok());
            assert_eq!(contract.markets(), [p1]);
            let event = decode_market_listed_event(get_emitted_events()[0].clone());
            assert_eq!(event.pool, p1);

            let p2 = AccountId::from([0x02; 32]);
            assert!(contract.support_market(p2).is_ok());
            assert_eq!(contract.markets(), [p1, p2]);
        }
    }
}
