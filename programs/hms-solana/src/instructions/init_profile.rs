use anchor_lang::prelude::*;
use crate::{
    constants::*,
    events::HealthProfileCreated,
    state::UserHealthProfile,
};

#[derive(Accounts)]
pub struct InitializeProfile<'info> {
    #[account(
        init,
        payer = user,
        space = UserHealthProfile::LEN,
        seeds = [USER_PROFILE_SEED, user.key().as_ref()],
        bump
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_profile(ctx: Context<InitializeProfile>) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    let user = &ctx.accounts.user;

    // Initialize the profile
    **profile = UserHealthProfile::new(user.key(), ctx.bumps.profile);

    // Emit event
    emit!(HealthProfileCreated {
        owner: user.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Health profile initialized for user: {}", user.key());
    Ok(())
}