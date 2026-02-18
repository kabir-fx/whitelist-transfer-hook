# Whitelist Transfer Hook

This example demonstrates how to implement a transfer hook using the SPL Token 2022 Transfer Hook interface to enforce whitelist restrictions on token transfers.

In this example, only whitelisted addresses will be able to transfer tokens that have this transfer hook enabled, providing fine-grained access control over token movements.

---

## Let's walk through the architecture:

For this program, we have 2 main state accounts:

1.  **Whitelist Account**: A global state account.
2.  **WhitelistEntry Account**: A PDA account for each whitelisted user.

### Whitelist State

```rust
#[account]
pub struct Whitelist {
    pub bump: u8,
}
```

### WhitelistEntry State

```rust
#[account]
pub struct WhitelistEntry {
    pub user_address: Pubkey,
    pub bump: u8,
}
```

Instead of storing a large vector of addresses in a single account (which has size limitations and high rent costs), we use a **Program Derived Address (PDA)** for each whitelisted user. This allows the whitelist to grow indefinitely without reallocation issues.

---

### The admin will be able to create new Whitelist accounts.

```rust
#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        seeds = [b"whitelist"],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}
```

- **admin**: The signer who initializes the whitelist.
- **whitelist**: The global state account derived from `b"whitelist"`.

---

### The admin will be able to manage whitelist operations (add/remove addresses):

#### Adding to Whitelist

```rust
#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        seeds = [b"whitelist", user.key().as_ref()],
        bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    // ...
}
```

When adding a user, we **initialize** a new `WhitelistEntry` account. The seeds are `b"whitelist"` and the `user`'s public key. The existence of this account signifies that the user is whitelisted.

#### Removing from Whitelist

```rust
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
```

When removing a user, we simply **close** the `WhitelistEntry` account and refund the rent to the admin.

---

### Token Minting with Transfer Hook

The program also provides an instruction to create a new token mint that automatically has the Transfer Hook extension enabled.

```rust
#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        mint::decimals = 9,
        mint::authority = user,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    // ...
}
```

This ensures that any token created via this instruction is immediately bound to the transfer hook logic of this program.

---

### The transfer hook will validate every token transfer:

```rust
#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    // ...

    #[account(
        // Here each owner must be whitelisted in order to make a transfer
        seeds = [b"whitelist", owner.key().as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
}
```

During a transfer, the Token 2022 program calls into our `transfer_hook` instruction. We validate the transfer by adding the `whitelist_entry` account as a required account in the context.

**Validation Logic:**

1.  We derive the address of the `whitelist_entry` account using the `owner` of the source token account.
2.  If the `whitelist_entry` account **does not exist**, the instruction will fail to deserialize the account, effectively blocking the transfer.
3.  If the account exists, the transfer is allowed to proceed.

This provides a secure, efficient, and gas-optimized way to enforce whitelist compliance on-chain.
