mod balance;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

use solana_program::pubkey::Pubkey;

solana_program::declare_id!("EjhMW84ENMdycHT2vtY8GdvvkcJrbZW6ohmvB72fLGqo");

// ----------------------------------------------------------------------------
// misc

pub const BASE_UNIT: u128 = 1000000000000000000;
pub const QUOTE_UNIT: u128 = 1000000000;

// ----------------------------------------------------------------------------
// state

pub fn get_state_address(payer_address: &Pubkey) -> Pubkey {
    get_state_address_with_seed(payer_address).0
}

pub fn get_state_address_with_seed(payer_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&payer_address.to_bytes(), br"state"], &id())
}

// ----------------------------------------------------------------------------
// mint

pub fn get_mint_address(payer_address: &Pubkey) -> Pubkey {
    get_mint_address_with_seed(payer_address).0
}

pub fn get_mint_address_with_seed(payer_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&payer_address.to_bytes(), br"mint"], &id())
}

// ----------------------------------------------------------------------------
// token pool

pub fn get_token_pool_address(payer_address: &Pubkey) -> Pubkey {
    get_token_pool_address_with_seed(payer_address).0
}

pub fn get_token_pool_address_with_seed(payer_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&payer_address.to_bytes(), br"token-pool"], &id())
}

// ----------------------------------------------------------------------------
// native pool

pub fn get_native_pool_address(payer_address: &Pubkey) -> Pubkey {
    get_native_pool_address_with_seed(payer_address).0
}

pub fn get_native_pool_address_with_seed(payer_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&payer_address.to_bytes(), br"native-pool"], &id())
}
