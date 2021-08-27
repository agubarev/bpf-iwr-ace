use crate::balance::Balance;
use crate::error::IWRError;
use crate::instruction::PoolInstruction;
use crate::state::State;
use crate::{
    get_mint_address, get_mint_address_with_seed, get_native_pool_address_with_seed,
    get_state_address_with_seed, get_token_pool_address, get_token_pool_address_with_seed, id,
    BASE_UNIT,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::sysvar::Sysvar;
use spl_associated_token_account::get_associated_token_address;
use spl_token::solana_program::program_option::COption;

// entrypoint! macro tells Solana that this function is an
// entry point for this program
entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = PoolInstruction::try_from_slice(instruction_data)?;
    let account_info_iter = &mut accounts.iter();

    match instruction {
        PoolInstruction::Initialize {
            total_token_supply,
            decimals,
            initial_quote_amount,
        } => {
            let payer_info = next_account_info(account_info_iter)?;
            let state_info = next_account_info(account_info_iter)?;
            let mint_info = next_account_info(account_info_iter)?;
            let token_pool_info = next_account_info(account_info_iter)?;
            let native_pool_info = next_account_info(account_info_iter)?;
            let system_program_info = next_account_info(account_info_iter)?;
            let token_program_info = next_account_info(account_info_iter)?;
            let rent_sysvar_info = next_account_info(account_info_iter)?;
            let rent = &Rent::from_account_info(rent_sysvar_info)?;

            // ----------------------------------------------------------------------------
            // state account

            let (state_address, state_bump_seed) = get_state_address_with_seed(payer_info.key);

            if state_address != *state_info.key {
                msg!("Error: state address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let state_signer_seeds: &[&[_]] =
                &[&payer_info.key.to_bytes(), br"state", &[state_bump_seed]];

            // ----------------------------------------------------------------------------
            // mint account

            let (mint_address, mint_bump_seed) = get_mint_address_with_seed(payer_info.key);

            if mint_address != *mint_info.key {
                msg!("Error: mint address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let mint_signer_seeds: &[&[_]] =
                &[&payer_info.key.to_bytes(), br"mint", &[mint_bump_seed]];

            // ----------------------------------------------------------------------------
            // token pool account

            let (token_pool_address, token_pool_bump_seed) =
                get_token_pool_address_with_seed(payer_info.key);

            if token_pool_address != *token_pool_info.key {
                msg!("Error: token pool address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let token_pool_signer_seeds: &[&[_]] = &[
                &payer_info.key.to_bytes(),
                br"token-pool",
                &[token_pool_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // native pool account

            let (native_pool_address, native_pool_bump_seed) =
                get_native_pool_address_with_seed(payer_info.key);

            if native_pool_address != *native_pool_info.key {
                msg!("Error: native pool address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let native_pool_signer_seeds: &[&[_]] = &[
                &payer_info.key.to_bytes(),
                br"native-pool",
                &[native_pool_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // processing

            let required_data_size = std::mem::size_of::<State>();

            msg!("creating state account");
            invoke_signed(
                &system_instruction::create_account(
                    payer_info.key,
                    &state_address,
                    1.max(Rent::default().minimum_balance(required_data_size)),
                    required_data_size as u64,
                    &program_id,
                ),
                &[
                    payer_info.clone(),
                    state_info.clone(),
                    system_program_info.clone(),
                    rent_sysvar_info.clone(),
                ],
                &[state_signer_seeds],
            )?;

            msg!("creating mint");
            invoke_signed(
                &system_instruction::create_account(
                    payer_info.key,
                    &mint_address,
                    1.max(rent.minimum_balance(spl_token::state::Mint::get_packed_len())),
                    spl_token::state::Mint::get_packed_len() as u64,
                    &spl_token::id(),
                ),
                &[
                    payer_info.clone(),
                    mint_info.clone(),
                    rent_sysvar_info.clone(),
                    system_program_info.clone(),
                ],
                &[mint_signer_seeds],
            )?;

            msg!("initializing mint");
            invoke(
                &spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    mint_info.key,
                    &token_pool_address,
                    None,
                    decimals,
                )?,
                &[
                    mint_info.clone(),
                    token_program_info.clone(),
                    rent_sysvar_info.clone(),
                ],
            )?;

            msg!("creating token pool account");
            invoke_signed(
                &system_instruction::create_account(
                    payer_info.key,
                    token_pool_info.key,
                    1.max(rent.minimum_balance(spl_token::state::Account::get_packed_len())),
                    spl_token::state::Account::get_packed_len() as u64,
                    &spl_token::id(),
                ),
                &[
                    payer_info.clone(),
                    token_pool_info.clone(),
                    system_program_info.clone(),
                ],
                &[token_pool_signer_seeds],
            )?;

            msg!("initializing token pool account");
            invoke(
                &spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    token_pool_info.key,
                    mint_info.key,
                    token_pool_info.key,
                )?,
                &[
                    token_pool_info.clone(),
                    token_program_info.clone(),
                    rent_sysvar_info.clone(),
                    mint_info.clone(),
                ],
            )?;

            msg!("minting tokens: {}", total_token_supply);
            invoke_signed(
                &spl_token::instruction::mint_to(
                    &spl_token::id(),
                    mint_info.key,
                    token_pool_info.key,
                    token_pool_info.key,
                    &[],
                    total_token_supply as u64,
                )?,
                &[
                    mint_info.clone(),
                    token_pool_info.clone(),
                    token_program_info.clone(),
                ],
                &[mint_signer_seeds, token_pool_signer_seeds],
            )?;

            msg!("creating native pool account");
            invoke_signed(
                &system_instruction::create_account(
                    payer_info.key,
                    native_pool_info.key,
                    1.max(Rent::default().minimum_balance(0)),
                    0,
                    &program_id,
                ),
                &[
                    payer_info.clone(),
                    native_pool_info.clone(),
                    system_program_info.clone(),
                    rent_sysvar_info.clone(),
                ],
                &[native_pool_signer_seeds],
            )?;

            msg!("funding native pool account");
            invoke(
                &system_instruction::transfer(
                    payer_info.key,
                    native_pool_info.key,
                    initial_quote_amount as u64,
                ),
                &[
                    payer_info.clone(),
                    native_pool_info.clone(),
                    system_program_info.clone(),
                ],
            )?;

            let initial_state = State {
                authority: *payer_info.key,
                mint_authority: *mint_info.key,
                base_pool_authority: *token_pool_info.key,
                quote_pool_authority: *native_pool_info.key,
                balance: Balance {
                    base: total_token_supply * BASE_UNIT,
                    quote: initial_quote_amount,
                },
            };

            state_info
                .data
                .borrow_mut()
                .copy_from_slice(&initial_state.try_to_vec().unwrap());
        }

        PoolInstruction::Buy(quote_amount) => {
            let authority_info = next_account_info(account_info_iter)?;
            let state_info = next_account_info(account_info_iter)?;
            let mint_info = next_account_info(account_info_iter)?;
            let token_pool_info = next_account_info(account_info_iter)?;
            let native_pool_info = next_account_info(account_info_iter)?;
            let beneficiary_info = next_account_info(account_info_iter)?;
            let customer_info = next_account_info(account_info_iter)?;
            let customer_token_associated_info = next_account_info(account_info_iter)?;
            let system_program_info = next_account_info(account_info_iter)?;
            let token_program_info = next_account_info(account_info_iter)?;

            // ----------------------------------------------------------------------------
            // state account

            let (state_address, state_bump_seed) = get_state_address_with_seed(authority_info.key);

            if state_address != *state_info.key {
                msg!("Error: state address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let state_signer_seeds: &[&[_]] = &[
                &authority_info.key.to_bytes(),
                br"state",
                &[state_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // mint account

            let (mint_address, mint_bump_seed) = get_mint_address_with_seed(authority_info.key);

            if mint_address != *mint_info.key {
                msg!("Error: mint address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let mint_signer_seeds: &[&[_]] =
                &[&authority_info.key.to_bytes(), br"mint", &[mint_bump_seed]];

            // ----------------------------------------------------------------------------
            // token pool account

            let (token_pool_address, token_pool_bump_seed) =
                get_token_pool_address_with_seed(authority_info.key);

            if token_pool_address != *token_pool_info.key {
                msg!("Error: token pool address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let token_pool_signer_seeds: &[&[_]] = &[
                &authority_info.key.to_bytes(),
                br"token-pool",
                &[token_pool_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // native pool account

            let (native_pool_address, native_pool_bump_seed) =
                get_native_pool_address_with_seed(authority_info.key);

            if native_pool_address != *native_pool_info.key {
                msg!("Error: native pool address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let native_pool_signer_seeds: &[&[_]] = &[
                &authority_info.key.to_bytes(),
                br"native-pool",
                &[native_pool_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // processing exchange

            let mut state: State = State::try_from_slice(*state_info.data.borrow())?;

            // making sure that customer account has enough lamports + rent buffer intact
            let customer_account_minimum_rent =
                &Rent::get()?.minimum_balance(customer_info.data_len());

            let quote_threshold = (quote_amount as u64 + customer_account_minimum_rent);

            // checking customer lamports
            if customer_info.lamports() < quote_threshold {
                msg!("not enough lamports");
                return Err(ProgramError::Custom(IWRError::NotEnoughLamports as u32));
            }

            // calculating base exchange amount
            let base_return = state.balance.calculate_base_for_quote_amount(quote_amount);
            let fee_amount = state.balance.calculate_fee_of(quote_amount);

            msg!(
                "exchanging {} lamports for {} base",
                quote_amount,
                base_return
            );

            msg!("debiting lamports: {} + fee {}", quote_amount, fee_amount);
            /*
             **customer_info.try_borrow_mut_lamports()? -= quote_amount as u64;
             **native_pool_info.try_borrow_mut_lamports()? += quote_amount as u64;
             **beneficiary_info.try_borrow_mut_lamports()? += fee_amount as u64;
             */

            invoke_signed(
                &system_instruction::transfer(
                    customer_info.key,
                    native_pool_info.key,
                    quote_amount as u64,
                ),
                &[
                    native_pool_info.clone(),
                    customer_info.clone(),
                    system_program_info.clone(),
                ],
                &[],
            )?;

            invoke_signed(
                &system_instruction::transfer(
                    customer_info.key,
                    beneficiary_info.key,
                    fee_amount as u64,
                ),
                &[
                    customer_info.clone(),
                    beneficiary_info.clone(),
                    system_program_info.clone(),
                ],
                &[],
            )?;

            msg!("crediting tokens: {}", base_return);
            invoke_signed(
                &spl_token::instruction::transfer(
                    &spl_token::id(),
                    token_pool_info.key,
                    customer_token_associated_info.key,
                    token_pool_info.key,
                    &[],
                    (base_return / BASE_UNIT) as u64,
                )?,
                &[
                    token_program_info.clone(),
                    token_pool_info.clone(),
                    customer_token_associated_info.clone(),
                    token_pool_info.clone(),
                    mint_info.clone(),
                ],
                &[mint_signer_seeds, token_pool_signer_seeds],
            )?;

            // applying changes to the balance
            state.balance.apply_buy_base_for_quote(quote_amount);

            state_info
                .data
                .borrow_mut()
                .copy_from_slice(&state.try_to_vec().unwrap());
        }

        PoolInstruction::Sell(base_amount) => {
            let authority_info = next_account_info(account_info_iter)?;
            let state_info = next_account_info(account_info_iter)?;
            let mint_info = next_account_info(account_info_iter)?;
            let token_pool_info = next_account_info(account_info_iter)?;
            let native_pool_info = next_account_info(account_info_iter)?;
            let beneficiary_info = next_account_info(account_info_iter)?;
            let customer_info = next_account_info(account_info_iter)?;
            let customer_token_associated_info = next_account_info(account_info_iter)?;
            let system_program_info = next_account_info(account_info_iter)?;
            let token_program_info = next_account_info(account_info_iter)?;

            // ----------------------------------------------------------------------------
            // state account

            let (state_address, state_bump_seed) = get_state_address_with_seed(authority_info.key);

            if state_address != *state_info.key {
                msg!("Error: state address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let state_signer_seeds: &[&[_]] = &[
                &authority_info.key.to_bytes(),
                br"state",
                &[state_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // mint account

            let (mint_address, mint_bump_seed) = get_mint_address_with_seed(authority_info.key);

            if mint_address != *mint_info.key {
                msg!("Error: mint address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let mint_signer_seeds: &[&[_]] =
                &[&authority_info.key.to_bytes(), br"mint", &[mint_bump_seed]];

            // ----------------------------------------------------------------------------
            // token pool account

            let (token_pool_address, token_pool_bump_seed) =
                get_token_pool_address_with_seed(authority_info.key);

            if token_pool_address != *token_pool_info.key {
                msg!("Error: token pool address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let token_pool_signer_seeds: &[&[_]] = &[
                &authority_info.key.to_bytes(),
                br"token-pool",
                &[token_pool_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // native pool account

            let (native_pool_address, native_pool_bump_seed) =
                get_native_pool_address_with_seed(authority_info.key);

            if native_pool_address != *native_pool_info.key {
                msg!("Error: native pool address derivation mismatch");
                return Err(ProgramError::InvalidArgument);
            }

            let native_pool_signer_seeds: &[&[_]] = &[
                &authority_info.key.to_bytes(),
                br"native-pool",
                &[native_pool_bump_seed],
            ];

            // ----------------------------------------------------------------------------
            // processing exchange

            let mut state: State = State::try_from_slice(*state_info.data.borrow())?;

            let quote_return = state.balance.calculate_quote_for_base_amount(base_amount);
            let fee_amount = state.balance.calculate_fee_of(quote_return);
            let net_quote_return = quote_return - fee_amount;

            msg!(
                "exchanging {} base for {} lamports",
                base_amount,
                quote_return
            );

            msg!("debiting tokens: {}", base_amount);
            invoke_signed(
                &spl_token::instruction::transfer(
                    &spl_token::id(),
                    customer_token_associated_info.key,
                    token_pool_info.key,
                    customer_info.key,
                    &[],
                    (base_amount / BASE_UNIT) as u64,
                )?,
                &[
                    token_program_info.clone(),
                    customer_token_associated_info.clone(),
                    token_pool_info.clone(),
                    customer_info.clone(),
                    mint_info.clone(),
                ],
                &[mint_signer_seeds, token_pool_signer_seeds],
            )?;

            msg!(
                "crediting lamports: {} - fee {} = {}",
                quote_return,
                fee_amount,
                quote_return - fee_amount
            );
            **native_pool_info.try_borrow_mut_lamports()? -= quote_return as u64;
            **customer_info.try_borrow_mut_lamports()? += net_quote_return as u64;
            **beneficiary_info.try_borrow_mut_lamports()? += fee_amount as u64;

            /*
            invoke_signed(
                &system_instruction::transfer(
                    native_pool_info.key,
                    customer_info.key,
                    net_quote_return as u64,
                ),
                &[
                    native_pool_info.clone(),
                    customer_info.clone(),
                    system_program_info.clone(),
                ],
                &[&native_pool_signer_seeds],
            )?;

            invoke_signed(
                &system_instruction::transfer(
                    native_pool_info.key,
                    beneficiary_info.key,
                    fee_amount as u64,
                ),
                &[
                    mint_info.clone(),
                    native_pool_info.clone(),
                    beneficiary_info.clone(),
                    system_program_info.clone(),
                ],
                &[&native_pool_signer_seeds],
            )?;
             */

            // applying changes to the balance
            state.balance.apply_sell_base(base_amount);

            state_info
                .data
                .borrow_mut()
                .copy_from_slice(&state.try_to_vec().unwrap());
        }
    }

    Ok(())
}
