/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by u128 (2**128 - 1).
  - JSON calls should pass u128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, LazyOption};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue, Promise};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    icos: UnorderedMap<u128, u128>, // new <price in Near, supply/offer>
    authorized_sellers: UnorderedMap<String, f64> // new <account_id, fee>
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
const YOCTO_NEAR: u128 = 1_000_000_000_000_000_000_000_000; // 1 $NEAR as yoctoNEAR

// Restrict some functions only to owner - new
fn assert_self() {
    assert_eq!(
        env::current_account_id(),
        env::predecessor_account_id(),
        "Can only be called by owner"
    );
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId, total_supply: u128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Example NEAR fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        total_supply: u128,
        metadata: FungibleTokenMetadata
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            icos: UnorderedMap::new(b"i".to_vec()),
            authorized_sellers: UnorderedMap::new(b"s".to_vec())
        };
        this.token.internal_register_account(owner_id.as_ref());
        this.token.internal_deposit(owner_id.as_ref(), total_supply.into());
        this
    }

    /// Create new ICO - new
    pub fn new_offer(&mut self, near_price: u128, supply: u128) {
        assert!(env::state_exists(), "Mint initial token supply");
        assert_self();
        let total_ico_supply: u128 = self.icos.values().sum();
        assert!(supply <= self.token.total_supply - total_ico_supply, "Not enough tokens for ICO");
        self.icos.insert(&near_price, &supply);
    }

    /// Remove ICO - new
    pub fn remove_offer(&mut self, near_price: u128) {
        assert!(env::state_exists(), "Mint initial token supply");
        assert_self();
        self.icos.remove(&near_price);
    }

    /// View ICO - new
    pub fn get_offer(&self, near_price: u128) -> Option<u128> {
        return self.icos.get(&near_price);
    }

    /// Get paginated ICOs - new
    pub fn get_all_offers(&self, from_index: u64, limit: u64) -> Vec<(u128, u128)> {
        let keys = self.icos.keys_as_vector();
        let values = self.icos.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, self.icos.len()))
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap()))
            .collect()
    }

    /// Add new authorized ICO tokens seller - new
    pub fn new_seller(&mut self, account_id: String, fee: f64) {
        assert_self();
        self.authorized_sellers.insert(&account_id, &fee);
    }

    /// Remove authorized seller - new
    pub fn remove_seller(&mut self, account_id: String) {
        assert_self();
        self.authorized_sellers.remove(&account_id);
    }

    /// View authorized ICO tokens seller - new
    pub fn get_seller(&self, account_id: String) -> Option<f64> {
        assert!(!self.authorized_sellers.get(&account_id).is_none(), "Seller/Exchange is not authorized");
        return self.authorized_sellers.get(&account_id);
    }

    /// Get all authorized ICO sellers paginated - new
    pub fn get_all_sellers(&self, from_index: u64, limit: u64) -> Vec<(String, f64)> {
        let keys = self.authorized_sellers.keys_as_vector();
        let values = self.authorized_sellers.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, self.authorized_sellers.len()))
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap()))
            .collect()
    }

    /// Check if ICO tokens buyer has storage - new
    pub fn has_storage(&self, account_id: ValidAccountId) -> bool {
        assert!(!self.storage_balance_of(account_id).is_none(), "Buyer has no storage balance");
        true
    }

    /// Transfer tokens via authorized ICO seller/exchange - new
    #[payable]
    pub fn transfer_tokens(&mut self, exchange_account: String, buyer_account_id: ValidAccountId, near_price: u128, tokens: u128, msg: String) -> u128 {
        assert!(env::state_exists(), "Mint initial token supply");
        assert!(!self.authorized_sellers.get(&exchange_account).is_none(), "Seller/Exchange is not authorized");
        assert!(!self.icos.get(&near_price).is_none(), "No ICO for this price");
        let total_ico_supply = self.icos.get(&near_price).unwrap();
        assert!(total_ico_supply > tokens, "Not enough tokens for this price");
        assert!(!self.storage_balance_of(buyer_account_id.clone()).is_none(), "Buyer has no storage balance");

        let total_price = tokens * near_price;
        let fee_percentage = self.authorized_sellers.get(&exchange_account).unwrap()/100.0;
        let fee = total_price as f64 * YOCTO_NEAR as f64 * fee_percentage;

        self.token.internal_transfer(
            &env::current_account_id(),
            &buyer_account_id.clone().to_string(),
            tokens,
            Some(msg)
        );

        self.icos.insert(&near_price, &(total_ico_supply - tokens));

        self.transfer_money(
            exchange_account,
            fee as u128,
        );

        return fee as u128;
    }

    pub fn transfer_money(&mut self, account_id: AccountId, amount: u128) {
        Promise::new(account_id).transfer(amount);
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }


}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
