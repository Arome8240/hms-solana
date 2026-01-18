use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::HealthRecordUpdated,
    state::{UserHealthProfile, HealthRecord, AccessGrant},
};

#[derive(Accounts)]
#[instruction(record_id: u64, metadata: String)]
pub struct UpdateRecord<'info> {
    #[account(
        seeds = [USER_PROFILE_SEED, record_owner.key().as_ref()],
        bump = profile.bump
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [HEALTH_RECORD_SEED, record_owner.key().as_ref(), record_id.to_le_bytes().as_ref()],
        bump = record.bump,
        constraint = record.owner == record_owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = record.is_accessible() @ HealthManagerError::RecordSoftDeleted
    )]
    pub record: Account<'info, HealthRecord>,

    /// CHECK: This is the owner of the record, validated in constraints
    pub record_owner: UncheckedAccount<'info>,

    #[account(mut)]
    pub actor: Signer<'info>,

    /// Optional access grant account for authorized users
    #[account(
        seeds = [ACCESS_GRANT_SEED, record_owner.key().as_ref(), actor.key().as_ref()],
        bump = access_grant.bump,
    )]
    pub access_grant: Option<Account<'info, AccessGrant>>,
}

pub fn update_record(
    ctx: Context<UpdateRecord>,
    _record_id: u64,
    metadata: String,
) -> Result<()> {
    let record = &mut ctx.accounts.record;
    let actor = &ctx.accounts.actor;
    let record_owner = &ctx.accounts.record_owner;

    // Check if actor is owner or has valid access
    if actor.key() != record_owner.key() {
        let access_grant = ctx.accounts.access_grant.as_ref()
            .ok_or(HealthManagerError::UnauthorizedAccess)?;

        require!(
            access_grant.has_permission(PERMISSION_WRITE),
            HealthManagerError::InsufficientPermissions
        );
    }

    // Update the record metadata
    record.update_metadata(metadata)?;

    // Emit event
    emit!(HealthRecordUpdated {
        owner: record_owner.key(),
        record_id: record.id,
        actor: actor.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Health record {} updated by: {}", record.id, actor.key());
    Ok(())
}