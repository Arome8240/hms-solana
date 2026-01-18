use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::HealthRecordAdded,
    state::{UserHealthProfile, HealthRecord},
};

#[derive(Accounts)]
#[instruction(record_type: String, encrypted_uri: String, data_hash: [u8; 32], metadata: String)]
pub struct AddRecord<'info> {
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
        space = HealthRecord::LEN,
        seeds = [HEALTH_RECORD_SEED, owner.key().as_ref(), profile.record_count.to_le_bytes().as_ref()],
        bump
    )]
    pub record: Account<'info, HealthRecord>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn add_record(
    ctx: Context<AddRecord>,
    record_type: String,
    encrypted_uri: String,
    data_hash: [u8; 32],
    metadata: String,
) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    let record = &mut ctx.accounts.record;
    let owner = &ctx.accounts.owner;

    // Create the health record
    **record = HealthRecord::new(
        profile.record_count,
        owner.key(),
        record_type.clone(),
        encrypted_uri,
        data_hash,
        metadata,
        ctx.bumps.record,
    )?;

    // Update profile
    profile.increment_record_count();

    // Emit event
    emit!(HealthRecordAdded {
        owner: owner.key(),
        record_id: record.id,
        record_type,
        actor: owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Health record {} added for user: {}", record.id, owner.key());
    Ok(())
}