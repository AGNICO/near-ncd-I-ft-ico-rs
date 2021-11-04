use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env,
    ext_contract,
    json_types::U128,
    near_bindgen,
    AccountId,
    Promise,
};

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Exchange {}

// One can provide a name, e.g. `ext` to use for generated methods.
#[ext_contract(ext)]
pub trait ExtExchange {
    fn merge_sort(&self, arr: Vec<u8>) -> PromiseOrValue<Vec<u8>>;
    fn merge(
        &self,
        #[callback_unwrap]
        #[serializer(borsh)]
        data0: Vec<u8>,
        #[callback_unwrap]
        #[serializer(borsh)]
        data1: Vec<u8>,
    ) -> Vec<u8>;
}

// If the name is not provided, the namespace for generated methods in derived by applying snake
// case to the trait name, e.g. ext_status_message.
#[ext_contract]
pub trait ExtFtIco {
    fn new_offer(&mut self, near_price: u128, supply: u128);
    fn remove_offer(&mut self, near_price: u128);
    fn get_offer(&self, near_price: u128) -> Option<u128>;
    fn get_all_offers(&self, from_index: u64, limit: u64) -> Vec<(u128, u128)>;
}

#[near_bindgen]
impl Exchange {
    pub fn deploy_ft_ico(&self, account_id: AccountId, amount: U128) {
        Promise::new(account_id)
            .create_account()
            .transfer(amount.0)
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(
                include_bytes!("../../CONTRACT-B-FT-ICO/res/fungible_token.wasm").to_vec(),
            );
    }

    // simple calls
    pub fn get_offer(&self, account_id: AccountId, near_price: u128) -> Promise {
        ext_ft_ico::get_offer(near_price, account_id, 0, env::prepaid_gas() / 2)
    }

    pub fn get_all_offers(&self, account_id: AccountId, from_index: u64, limit: u64) -> Promise {
        ext_ft_ico::get_all_offers(from_index, limit, account_id, 0, env::prepaid_gas() / 2)
    }

    // complex call - buying/selling ICO tokens
    /* pub fn new_offer(&mut self, account_id: AccountId, near_price: u128, supply: u128, from_index: u64, limit: u64) -> Promise {
        let prepaid_gas = env::prepaid_gas();
        ext_ft_ico::new_offer(near_price, supply, account_id.clone(), 0, prepaid_gas / 3).then(
            ext_ft_ico::get_all_offers(
                from_index,
                limit,
                account_id,
                0,
                prepaid_gas / 3,
            ),
        )
    } */

    pub fn transfer_money(&mut self, account_id: AccountId, amount: u64) {
        Promise::new(account_id).transfer(amount as u128);
    }
}