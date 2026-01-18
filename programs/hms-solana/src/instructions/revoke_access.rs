use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::AccessRevoked,
    state::{UserHealthProfile, AccessGrant},
};

#[derive(Accounts)]
#[instruction(grantee: Pubkey)]
pub struct RevokeAccess<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        close = owner,
        seeds = [ACCESS_GRANT_SEED, owner.key().as_ref(), grantee.as_ref()],
        bump = access_grant.bump,
        constraint = access_grant.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub access_grant: Account<'info, AccessGrant>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn revoke_access(
    ctx: Context<RevokeAccess>,
    grantee: Pubkey,
) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event before closing the account
    emit!(AccessRevoked {
        owner: owner.key(),
        grantee,
        actor: owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Access revoked for {} by owner: {}", grantee, owner.key());

    // Account will be closed automatically due to close constraint
    Ok(())
}