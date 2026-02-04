use anchor_lang::prelude::*;

#[account]
pub struct Whitelist {
    pub bump: u8,
}
