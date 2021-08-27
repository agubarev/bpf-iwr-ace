// #![cfg(feature = "test-bpf")]

use borsh::BorshDeserialize;
use bpf_iwr_ace::processor::process_instruction;
use bpf_iwr_ace::state::State;
use bpf_iwr_ace::{
    get_mint_address, get_native_pool_address, get_state_address, get_token_pool_address, id,
    instruction,
};
use bpf_iwr_ace::{BASE_UNIT, QUOTE_UNIT};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_sdk::account::Account;
use solana_sdk::signature::Keypair;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;
use std::borrow::Borrow;
use {
    solana_program::system_program,
    solana_program_test::*,
    solana_sdk::{signature::Signer, transaction::Transaction},
};

#[tokio::test]
// #[cfg(feature = "test-bpf")]
async fn test_full_cycle() -> Result<(), ProgramError> {
    let program_id = id();

    let customer = Keypair::new();
    let beneficiary = Keypair::new();

    let mut program_test =
        ProgramTest::new("bpf_iwr_ace", program_id, processor!(process_instruction));

    program_test.add_account(
        customer.pubkey().clone(),
        Account {
            lamports: (QUOTE_UNIT * 5) as u64,
            ..Account::default()
        },
    );

    program_test.add_account(
        beneficiary.pubkey().clone(),
        Account {
            lamports: 1,
            ..Account::default()
        },
    );

    let (mut banks_client, authority, recent_blockhash) = program_test.start().await;

    let state_address = get_state_address(&authority.pubkey());
    let mint_address = get_mint_address(&authority.pubkey());
    let token_pool_address = get_token_pool_address(&authority.pubkey());
    let native_pool_address = get_native_pool_address(&authority.pubkey());
    let customer_associated_token_address =
        get_associated_token_address(&customer.pubkey(), &mint_address);

    println!("authority = {:#?}", authority.pubkey());
    println!("state = {:#?}", state_address);
    println!("mint = {:#?}", mint_address);
    println!("token pool = {:#?}", token_pool_address);
    println!("native pool = {:#?}", native_pool_address);
    println!("beneficiary = {:#?}", beneficiary.pubkey());
    println!("customer = {:#?}", customer.pubkey());
    println!(
        "customer associated token address = {:#?}",
        customer_associated_token_address
    );
    println!("\n");

    // ----------------------------------------------------------------------------
    // initializing pool

    let mut tx = Transaction::new_with_payer(
        &[
            instruction::initialize_pool(
                &authority.pubkey(),
                &state_address,
                &mint_address,
                &token_pool_address,
                &native_pool_address,
                1000000,
                18,
                QUOTE_UNIT,
            ),
            spl_associated_token_account::create_associated_token_account(
                &customer.pubkey(),
                &customer.pubkey(),
                &mint_address,
            ),
            instruction::buy(
                &authority.pubkey(),
                &state_address,
                &mint_address,
                &token_pool_address,
                &native_pool_address,
                &beneficiary.pubkey(),
                &customer.pubkey(),
                &get_associated_token_address(&customer.pubkey(), &mint_address),
                QUOTE_UNIT,
            ),
            instruction::sell(
                &authority.pubkey(),
                &state_address,
                &mint_address,
                &token_pool_address,
                &native_pool_address,
                &beneficiary.pubkey(),
                &customer.pubkey(),
                &get_associated_token_address(&customer.pubkey(), &mint_address),
                BASE_UNIT * 500000,
            ),
        ],
        Some(&authority.pubkey()),
    );

    tx.sign(&[&authority, &customer], recent_blockhash);
    banks_client.process_transaction(tx).await.unwrap();

    // ----------------------------------------------------------------------------
    // validation

    let token_pool_account = banks_client.get_account(token_pool_address).await?.unwrap();
    let token_pool_state = TokenAccount::unpack_from_slice(token_pool_account.data.borrow())?;
    println!("token pool account = {:#?}\n", token_pool_account);
    println!("token pool token state = {:#?}\n", token_pool_state);

    let customer_account = banks_client.get_account(customer.pubkey()).await?.unwrap();
    let customer_token_associated_account = banks_client
        .get_account(customer_associated_token_address)
        .await?
        .unwrap();
    let customer_token_state =
        TokenAccount::unpack_from_slice(customer_token_associated_account.data.borrow())?;
    println!("customer account = {:#?}\n", customer_account);
    println!("customer token state = {:#?}\n", customer_token_state);

    let native_pool_account = banks_client
        .get_account(native_pool_address)
        .await?
        .unwrap();
    println!(
        "native pool account after exchange = {:#?}\n",
        native_pool_account
    );

    let beneficiary_account = banks_client
        .get_account(beneficiary.pubkey())
        .await?
        .unwrap();
    println!(
        "beneficiary account after exchange = {:#?}\n",
        beneficiary_account
    );

    assert_eq!(native_pool_account.lamports, 0);

    Ok(())
}
