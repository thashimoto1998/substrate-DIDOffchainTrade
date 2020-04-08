#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
    pub trait DIDOffchainTradeApi {
        fn get_nonce() -> i32;
        fn get_status() -> i32;
        fn get_owner() -> u64;
        fn get_grantee() -> u64;
        fn get_did_key() -> i32;
        fn access_condition_address_key() -> i32;
    }
}

