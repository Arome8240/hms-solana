use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::AccessGranted,
    state::{UserHealthProfile, AccessGrant},
};

#[derive(Accounts)]
#[instruction(grantee: Pubkey, expires_at: i64, permissions: u8)]
pub struct GrantAccess<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        init,
        payer = owner,
        space = AccessGrant::LEN,
        seeds = [ACCESS_GRANT_SEED, owner.key().as_ref(), grantee.as_ref()],
        bump
    )]
    pub access_grant: Account<'info, AccessGrant>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn grant_access(
    ctx: Context<GrantAccess>,
    grantee: Pubkey,
    expires_at: i64,
    permissions: u8,
) -> Result<()> {
    let access_grant = &mut ctx.accounts.access_grant;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Validate that owner is not granting access to themselves
    require!(
        grantee != owner.key(),
        HealthManagerError::CannotGrantAccessToSelf
    );

    // Create the access grant
    **access_grant = AccessGrant::new(
        owner.key(),
        grantee,
        expires_at,
        permissions,
        ctx.bumps.access_grant,
    )?;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(AccessGranted {
        owner: owner.key(),
        grantee,
        permissions,
        expires_at,
        actor: owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Access granted to {} by owner: {}", grantee, owner.key());
    Ok(())
}