use std::str::FromStr;

use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{prelude::*, InstructionData};
use tuktuk_program::{
    compile_transaction,
    tuktuk::{
        cpi::{
            accounts::{InitializeTaskQueueV0, QueueTaskV0},
            initialize_task_queue_v0, queue_task_v0,
        },
        program::Tuktuk,
        types::TriggerV0,
    },
    types::QueueTaskArgsV0,
    TransactionSourceV0,
};

use crate::state::Whitelist;

#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(
        mut,
        address = Pubkey::from_str("Bm3PBYtPwN17nkSvsogNgk1ycLBgf3BTNpwnrNzZnMRp").unwrap()
    )]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + 1, // 8 bytes for discriminator, 1 byte for bump
        seeds = [b"whitelist"],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    #[account(mut)]
    /// CHECK: Don't need to parse this account, just using it in CPI
    pub task_queue: UncheckedAccount<'info>,
    /// CHECK: Don't need to parse this account, just using it in CPI
    pub task_queue_authority: UncheckedAccount<'info>,
    /// CHECK: Initialized in CPI
    #[account(mut)]
    pub task: UncheckedAccount<'info>,
    /// CHECK: Via seeds
    #[account(
        mut,
        seeds = [b"queue_authority"],
        bump
    )]
    pub queue_authority: AccountInfo<'info>,
    /// CHECK: Automatically initialized by the user in this transaction
    #[account(
        mut,
        seeds = [b"whitelist", user.key().as_ref()],
        bump
    )]
    pub whitelist_entry: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub tuktuk_program: Program<'info, Tuktuk>,
}

impl<'info> Schedule<'info> {
    pub fn schedule(&mut self, task_id: u16, bumps: ScheduleBumps) -> Result<()> {
        let (compiled_tx, _) = compile_transaction(
            vec![Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(self.task.key(), true), // Task is admin and payer
                    AccountMeta::new(self.whitelist.key(), false),
                    AccountMeta::new(self.whitelist_entry.key(), false),
                    AccountMeta::new_readonly(self.system_program.key(), false),
                ],
                data: crate::instruction::AddToWhitelist {
                    user: self.user.key(),
                }
                .data(),
            }],
            vec![],
        )
        .unwrap();

        // Transfer lamports to the task account so it can pay for the whitelist_entry initialization rent
        let rent = Rent::get()?.minimum_balance(8 + 32 + 1);
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &self.user.key(),
                &self.task.key(),
                rent,
            ),
            &[
                self.user.to_account_info(),
                self.task.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        queue_task_v0(
            CpiContext::new_with_signer(
                self.tuktuk_program.to_account_info(),
                QueueTaskV0 {
                    payer: self.user.to_account_info(),
                    queue_authority: self.queue_authority.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task_queue_authority: self.task_queue_authority.to_account_info(),
                    task: self.task.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
                &[&["queue_authority".as_bytes(), &[bumps.queue_authority]]],
            ),
            QueueTaskArgsV0 {
                trigger: TriggerV0::Now,
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: Some(1000002),
                free_tasks: 1,
                id: task_id,
                description: "test".to_string(),
            },
        )?;

        Ok(())
    }
}
