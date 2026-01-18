use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::HealthRecordDeleted,
    state::{UserHealthProfile, HealthRecord},
};

#[derive(Accounts)]
#[instruction(record_id: u64)]
pub struct DeleteRecord<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [HEALTH_RECORD_SEED, owner.key().as_ref(), record_id.to_le_bytes().as_ref()],
        bump = record.bump,
        constraint = record.owner == owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = record.is_accessible() @ HealthManagerError::RecordSoftDeleted
    )]
    pub record: Account<'info, HealthRecord>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn delete_record(
    ctx: Context<DeleteRecord>,
    _record_id: u64,
) -> Result<()> {
    let record = &mut ctx.accounts.record;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Soft delete the record
    record.soft_delete();

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(HealthRecordDeleted {
        owner: owner.key(),
        record_id: record.id,
        actor: owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Health record {} soft deleted by owner: {}", record.id, owner.key());
    Ok(())
}