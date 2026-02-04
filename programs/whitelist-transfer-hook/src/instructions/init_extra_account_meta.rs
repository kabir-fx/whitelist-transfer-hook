use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};

use crate::ID;

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas()?.len()
        ).unwrap(),
        payer = payer
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        // Derive the whitelist PDA using our program ID
        let (whitelist_pda, _bump) = Pubkey::find_program_address(&[b"whitelist"], &ID);

        let whitelist_entry_seeds = &[
            spl_tlv_account_resolution::seeds::Seed::Literal {
                bytes: b"whitelist".to_vec(),
            },
            // Take the Public Key of the account at Index 3 (which is the Owner), and use it as a seed to derive the address of the whitelist_entry PDA.
            spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 3 },
        ];

        Ok(vec![
            ExtraAccountMeta::new_with_pubkey(&whitelist_pda.to_bytes().into(), false, false)
                .unwrap(),
            ExtraAccountMeta::new_with_seeds(whitelist_entry_seeds, false, false)
                .unwrap(),
        ])
    }
}
