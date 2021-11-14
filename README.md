# NEAR Fungible Token ICO Exchange

[image]

[video 1]

## ⚠️ Warning

This is DEMO only! This app was built only for demonstration purposes during Near Certified Developer II course.

Any content produced by NEAR, or developer resources that NEAR provides, are for educational and inspiration purposes only. NEAR does not encourage, induce or sanction the deployment of any such applications in violation of applicable laws or regulations.

## Contract A - EXCHANGE contract

```ts
/**
 * Cross contract call method with 3 callbacks:
 * 1) It checks whether the Seller/Exchange contract (A) is authorized in FT ICO contract (B) or not
 * 2) It checks whether the Buyer account has enough storage balance in FT ICO contract (B) to be able to buy FT or not
 * 3) It calls transfer_tokens method in FT ICO contract (B), which transfers FTs to the Buyer account and sends calculated reward/fee back to the Seller/Exchange (A)
 *
 * If all 3 callbacks are successful, the buy_tokens method is called and Seller/Exchange contract(A) sends money for tokens to the FT ICO ICO contract (B)
 */
pub fn transfer_tokens(&self, ico_account_id: AccountId, buyer_account_id:AccountId, near_price: u128, tokens: u128, msg: String) -> Promise

/**
 * Callback method for transfer_tokens cross contract call.
 *
 * If successful, Seller/Exchange contract(A) transfers money (NEAR) for fungible tokens to the FT ICO ICO contract (B).
 * This method returns a reward/fee, which is sent back from FT ICO contract (b) to the Seller/Exchange contract (A) as a profit.
 */
pub fn buy_tokens(&mut self, ico_account_id: AccountId, amount: u128) -> u128

/**
 * Helper method - money (NEAR) transfer to another account
 */
pub fn transfer_money(&mut self, account_id: AccountId, amount: u64)
```

## Contract B - Fungible Token ICO contract

```ts
/**
 * Initialization of the Fungible Token contract with [FT metadata (NEP-148)](https://nomicon.io/Standards/FungibleToken/Metadata.html#reference-level-explanation){:target="_blank"}
 */
pub fn new(owner_id: ValidAccountId, total_supply: u128, metadata: FungibleTokenMetadata) -> Self

/**
 * Create new FT ICO offer with predefined supply and price in NEAR
 */
pub fn new_offer(&mut self, near_price: u128, supply: u128)

/**
 * Remove FT ICO offer
 */
pub fn remove_offer(&mut self, near_price: u128)

/**
 * View single FT ICO offer
 */
pub fn get_offer(&self, near_price: u128) -> Option<u128>

/**
 * Get all FT ICO offers - paginated
 */
pub fn get_all_offers(&self, from_index: u64, limit: u64) -> Vec<(u128, u128)>

/**
 * Add new authorized Seller/Exchange with predefined fee/reward (percentage from every FT sale)
 */
pub fn new_seller(&mut self, account_id: String, fee: f64)

/**
 * Remove/Unauthorize Seller/Exchange
 */
pub fn remove_seller(&mut self, account_id: String)

/**
 * Get authorized Seller/Exchange and it's fee/reward
 */
pub fn get_seller(&self, account_id: String) -> Option<f64>

/**
 * Get all authorized Sellers/Exchanges - paginated
 */
pub fn get_all_sellers(&self, from_index: u64, limit: u64) -> Vec<(String, f64)>

/**
 * Helper method to check whether the ICO tokens buyer has a storage
 */
pub fn has_storage(&self, account_id: ValidAccountId) -> bool

/**
 * Main method for transfering FT ICO tokens to a buyer and paying fee/reward to a Seller/Exchange
 */
pub fn transfer_tokens(&mut self, exchange_account: String, buyer_account_id: ValidAccountId, near_price: u128, tokens: u128, msg: String) -> u128

/**
 * Helper method - money (NEAR) transfer to another account
 */
pub fn transfer_money(&mut self, account_id: AccountId, amount: u64)

```


## Usage

### Build and Dev deploy contracts

`sh`

`./build.sh`

`cd contract-a-exchange`

`near dev-deploy --wasmFile res/contract_a_exchange.wasm`

`source neardev/dev-account.env`

`export EXCHANGE=$CONTRACT_NAME`

The Seller/Exchange contract dev-account should be the same as `$CONTRACT_NAME`

`echo $EXCHANGE $CONTRACT_NAME`


`cd ../contract-b-ft-ico`

`near dev-deploy --wasmFile res/fungible_token.wasm`

`source neardev/dev-account.env`

`export ICO=$CONTRACT_NAME`

The FT ICO contract dev-account should be the same as `$CONTRACT_NAME`

`echo $ICO $CONTRACT_NAME`

### Initialize the FT ICO contract and get the fungible token metadata

`near call $ICO new '{"owner_id": "'$ICO'", "total_supply": "1000", "metadata": { "spec": "ft-1.0.0", "name": "NCD Token", "symbol": "NCDT", "decimals": 2 }}' --accountId $ICO`

`near view $ICO ft_metadata`
