use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum CrowdFundingInstruction {
    /// Enable Crowd Funding Campaign instruction
    ///
    /// Accounts
    /// 1 `[w]`  Campaign Account
    /// 2 `[ ]`  Campaign Authority
    /// 3 `[s]`  Campaign Creator
    /// 4 `[ ]`  Campaign Creator's USDC Token Account
    /// 5 `[ ]`  Fee Vault
    /// 6 `[ ]`  Token Program
    /// 7 `[ ]`  Crowd Funding Program
    EnableCampaign { amount: u64 },
}

pub fn ix_pay_enable_fee(
    fund_id: &Pubkey,
    authority: &Pubkey,
    creator: &Pubkey,
    creator_token_account: &Pubkey,
    fee_vault: &Pubkey,
    token_program_id: &Pubkey,
    fund_program_id: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*fund_id, false),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new(*creator, true),
        AccountMeta::new(*creator_token_account, false),
        AccountMeta::new(*fee_vault, false),
        AccountMeta::new_readonly(*token_program_id, false),
    ];
    Instruction {
        program_id: *fund_program_id,
        accounts,
        data: CrowdFundingInstruction::EnableCampaign { amount }
            .try_to_vec()
            .unwrap(),
    }
}
