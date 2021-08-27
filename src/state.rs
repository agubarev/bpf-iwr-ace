// Based on `record` program state from the solana-program-library
use crate::balance::Balance;
use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{program_pack::IsInitialized, pubkey::Pubkey},
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct State {
    pub authority: Pubkey,
    pub mint_authority: Pubkey,
    pub base_pool_authority: Pubkey,
    pub quote_pool_authority: Pubkey,
    pub balance: Balance,
}
