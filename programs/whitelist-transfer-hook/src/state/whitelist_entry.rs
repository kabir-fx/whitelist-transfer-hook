use anchor_lang::prelude::*;

// New PDA that will store user address and it's calculated bump (so that we don't have to cal. it again ad again)
#[account]
pub struct WhitelistEntry {
    pub user_address: Pubkey,
    pub bump: u8,
}
