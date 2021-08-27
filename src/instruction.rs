use crate::id;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum PoolInstruction {
    Initialize {
        total_token_supply: u128,
        decimals: u8,
        initial_quote_amount: u128,
    },
    Buy(u128),
    Sell(u128),
}

pub fn initialize_pool(
    authority_address: &Pubkey,
    state_address: &Pubkey,
    mint_address: &Pubkey,
    token_pool_address: &Pubkey,
    native_pool_address: &Pubkey,
    total_token_supply: u128,
    decimals: u8,
    initial_quote_amount: u128,
) -> Instruction {
    Instruction::new_with_borsh(
        id(),
        &PoolInstruction::Initialize {
            total_token_supply,
            decimals,
            initial_quote_amount,
        },
        vec![
            AccountMeta::new(*authority_address, true),
            AccountMeta::new(*state_address, false),
            AccountMeta::new(*mint_address, false),
            AccountMeta::new(*token_pool_address, false),
            AccountMeta::new(*native_pool_address, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
    )
}

pub fn buy(
    authority_address: &Pubkey,
    state_address: &Pubkey,
    mint_address: &Pubkey,
    token_pool_address: &Pubkey,
    native_pool_address: &Pubkey,
    beneficiary_address: &Pubkey,
    customer_address: &Pubkey,
    customer_token_associated_address: &Pubkey,
    quote_amount: u128,
) -> Instruction {
    Instruction::new_with_borsh(
        id(),
        &PoolInstruction::Buy(quote_amount),
        vec![
            AccountMeta::new(*authority_address, true),
            AccountMeta::new(*state_address, false),
            AccountMeta::new(*mint_address, false),
            AccountMeta::new(*token_pool_address, false),
            AccountMeta::new(*native_pool_address, false),
            AccountMeta::new(*beneficiary_address, false),
            AccountMeta::new(*customer_address, true),
            AccountMeta::new(*customer_token_associated_address, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
    )
}

pub fn sell(
    authority_address: &Pubkey,
    state_address: &Pubkey,
    mint_address: &Pubkey,
    token_pool_address: &Pubkey,
    native_pool_address: &Pubkey,
    beneficiary_address: &Pubkey,
    customer_address: &Pubkey,
    customer_token_associated_address: &Pubkey,
    base_amount: u128,
) -> Instruction {
    Instruction::new_with_borsh(
        id(),
        &PoolInstruction::Sell(base_amount),
        vec![
            AccountMeta::new(*authority_address, true),
            AccountMeta::new(*state_address, false),
            AccountMeta::new(*mint_address, false),
            AccountMeta::new(*token_pool_address, false),
            AccountMeta::new(*native_pool_address, false),
            AccountMeta::new(*beneficiary_address, false),
            AccountMeta::new(*customer_address, false),
            AccountMeta::new(*customer_token_associated_address, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
    )
}
