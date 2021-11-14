use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env,
    ext_contract,
    near_bindgen,
    AccountId,
    Promise,
    PromiseResult
};

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Exchange {}

const YOCTO_NEAR: u128 = 1_000_000_000_000_000_000_000_000; // 1 $NEAR as yoctoNEAR

#[ext_contract(ext)]
pub trait ExtExchange {
    fn buy_tokens(&self, ico_account_id: AccountId, amount: u128) -> String;
}

#[ext_contract]
pub trait ExtFtIco {
    fn get_seller(&self, account_id: String);
    fn transfer_tokens(&mut self, exchange_account: String, buyer_account_id: AccountId, near_price: u128, tokens: u128, msg: String) -> f64;
    fn has_storage(&self, account_id: AccountId) -> bool;
}

#[near_bindgen]
impl Exchange {
    /// TO DO: quite interesting - possible improvement: Exchange can deploy complete ICO for some contract/account
    /* pub fn deploy_ft_ico(&self, account_id: AccountId, amount: U128) {
        Promise::new(account_id)
            .create_account()
            .transfer(amount.0)
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(
                include_bytes!("../../CONTRACT-B-FT-ICO/res/fungible_token.wasm").to_vec(),
            );
    } */

    pub fn buy_tokens(&mut self, ico_account_id: AccountId, amount: u128) -> u128 {
        assert_eq!(env::promise_results_count(), 3, "This is a callback method");

        // check if the seller/exchange is authorized in FT ICO contract
        let _authorized_seller: f64 = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic_str("Seller/Exchange is not authorized"),
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<f64>(&result)
                .unwrap()
                .into(),
        };

        // check if buyer has FT storage
        let _has_storage_balance: bool = match env::promise_result(1) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic_str("Failed to check storage balance"),
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<bool>(&result)
                .unwrap()
                .into(),
        };

        // transfer tokens
        let fee: u128 = match env::promise_result(2) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic_str("Failed to transfer tokens"),
            PromiseResult::Successful(result) => {
                self.transfer_money(
                    ico_account_id.clone(),
                    amount,
                );

                near_sdk::serde_json::from_slice::<u128>(&result)
                .unwrap()
                .into()
            }
        };

        fee // returns fee, which is sent back from ICO contract to Exchange contract as profit
    }

    pub fn transfer_tokens(&self, ico_account_id: AccountId, buyer_account_id:AccountId, near_price: u128, tokens: u128, msg: String) -> Promise {
        let available_balance = env::account_balance() + env::account_locked_balance();
        assert!(available_balance > near_price * tokens * YOCTO_NEAR + 5*YOCTO_NEAR, "No available balance to finish the transaction: {}", available_balance.to_string());
        let prepaid_gas = env::prepaid_gas();
        ext_ft_ico::get_seller(
            env::current_account_id().to_string(), // function parameter ('account_id')
            ico_account_id.clone(), // id of contract account
            0,
            prepaid_gas/5
        )
        .and(ext_ft_ico::has_storage(
            buyer_account_id.clone(),
            ico_account_id.clone(),
            0,
            prepaid_gas/5
        ))
        .and(ext_ft_ico::transfer_tokens(
            env::current_account_id().to_string(),
            buyer_account_id.clone(),
            near_price,
            tokens,
            msg,
            ico_account_id.clone(),
            0,
            prepaid_gas/5
        ))
        .then(ext::buy_tokens(
            ico_account_id.clone(),
            near_price * tokens * YOCTO_NEAR,
            env::current_account_id(),
            0,
            prepaid_gas/5
        ))
    }

    pub fn transfer_money(&mut self, account_id: AccountId, amount: u128) {
        Promise::new(account_id).transfer(amount);
    }
}