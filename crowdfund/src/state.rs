use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const FEE: u64 = 50000000; // 50 USDC to enable the Crowd Funding Campaign

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, BorshSchema)]
pub struct CampaignAccount {
    pub enabled: u8,

    pub nonce: u8,

    pub goal_amount: u64,

    pub creator: Pubkey,

    pub fee_vault: Pubkey,
}
