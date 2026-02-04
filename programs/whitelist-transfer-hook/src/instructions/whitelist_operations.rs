use anchor_lang::prelude::*;

use crate::state::{whitelist::Whitelist, WhitelistEntry};

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    // Even though whitelist account is not being used anywhere in the operation - it will be a good practice to ensure that it exists before an account is added as a whitelist
    #[account(
        mut,
        seeds = [b"whitelist"],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,

    // Whitelist entry for each individual user - derived from "whitelist" and user's pub key
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 1,
        seeds = [b"whitelist", user.key().as_ref()],
        bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, user: Pubkey, bumps: AddToWhitelistBumps) -> Result<()> {
        // Populate the fields
        self.whitelist_entry.user_address = user;
        self.whitelist_entry.bump = bumps.whitelist_entry;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"whitelist", user.key().as_ref()],
        bump = whitelist_entry.bump,
        close = admin,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self, _user: Pubkey) -> Result<()> {
        // Closing an account will automatically remove whitelist entry for the user
        
        Ok(())
    }
}
