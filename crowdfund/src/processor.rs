use crate::{
    error::CrowdError,
    instruction::CrowdFundingInstruction,
    state::{CampaignAccount, FEE},
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

use spl_token::{instruction::TokenInstruction, state::Account as TokenAccount};

use borsh::{BorshDeserialize, BorshSerialize};

pub struct Processor {}

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = CrowdFundingInstruction::try_from_slice(instruction_data)?;
        msg!("The instruction data is {:#?}", instruction);

        match instruction {
            CrowdFundingInstruction::EnableCampaign { amount } => {
                msg!("Enabling the Fund");
                Self::process_enable_fund(accounts, amount, program_id)
            }
        }
    }

    pub fn process_enable_fund(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let accounts_info_iter = &mut accounts.iter();

        let campaign_ai = next_account_info(accounts_info_iter)?;
        let authority_ai = next_account_info(accounts_info_iter)?;
        let creator_ai = next_account_info(accounts_info_iter)?;
        let creators_token_ai = next_account_info(accounts_info_iter)?;
        let usdc_fee_vault_ai = next_account_info(accounts_info_iter)?;
        let token_program_ai = next_account_info(accounts_info_iter)?;

        let mut fund_data =
            try_from_slice_unchecked::<CampaignAccount>(&campaign_ai.data.borrow())?;

        if fund_data.enabled == 1 {
            return Err(CrowdError::AlreadyEnabled.into());
        }

        if !creator_ai.is_signer {
            return Err(CrowdError::InvalidSignature.into());
        }

        if *creator_ai.key != fund_data.creator {
            return Err(CrowdError::CreatorMismatch.into());
        }

        if *authority_ai.key != Self::authority_id(program_id, campaign_ai.key, fund_data.nonce)? {
            return Err(CrowdError::InvalidAuthority.into());
        }

        if amount != FEE {
            return Err(CrowdError::InvalidAmount.into());
        }

        let usdc_fee_vault_owner =
            TokenAccount::unpack_from_slice(&usdc_fee_vault_ai.try_borrow_data()?)?.owner;

        if usdc_fee_vault_owner != *authority_ai.key {
            return Err(CrowdError::InvalidFeeAccount.into());
        }

        Self::token_transfer(
            campaign_ai.key,
            token_program_ai.clone(),
            creators_token_ai.clone(),
            usdc_fee_vault_ai.clone(),
            creator_ai.clone(),
            fund_data.nonce,
            amount,
        )?;

        fund_data.enabled = 1;

        fund_data
            .serialize(&mut *campaign_ai.data.borrow_mut())
            .map_err(|e| e.into())
    }

    pub fn token_transfer<'a>(
        campaign_account_seed: &Pubkey,
        token_program: AccountInfo<'a>,
        source: AccountInfo<'a>,
        destination: AccountInfo<'a>,
        authority: AccountInfo<'a>,
        nonce: u8,
        amount: u64,
    ) -> Result<(), ProgramError> {
        let campaign_bytes = campaign_account_seed.to_bytes();

        let authority_signature_seeds = [&campaign_bytes[..32], &[nonce]];

        let signers = &[&authority_signature_seeds[..]];

        let data = TokenInstruction::Transfer { amount }.pack();

        let mut accounts = Vec::with_capacity(3);

        accounts.push(AccountMeta::new(*source.key, false));

        accounts.push(AccountMeta::new(*destination.key, false));

        accounts.push(AccountMeta::new_readonly(*authority.key, true));

        let ix = Instruction {
            program_id: *token_program.key,
            accounts,
            data,
        };

        invoke_signed(
            &ix,
            &[source, destination, authority, token_program],
            signers,
        )
    }

    pub fn authority_id(
        program_id: &Pubkey,
        seed_pubkey: &Pubkey,
        nonce: u8,
    ) -> Result<Pubkey, CrowdError> {
        Pubkey::create_program_address(&[&seed_pubkey.to_bytes()[..32], &[nonce]], program_id)
            .or(Err(CrowdError::InvalidAuthority))
    }
}
