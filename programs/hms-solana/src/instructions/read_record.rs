use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::AuthorizedRecordAccess,
    state::{UserHealthProfile, HealthRecord, AccessGrant},
};

#[derive(Accounts)]
#[instruction(record_id: u64)]
pub struct ReadRecord<'info> {
    #[account(
        seeds = [USER_PROFILE_SEED, record_owner.key().as_ref()],
        bump = profile.bump
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        seeds = [HEALTH_RECORD_SEED, record_owner.key().as_ref(), record_id.to_le_bytes().as_ref()],
        bump = record.bump,
        constraint = record.owner == record_owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = record.is_accessible() @ HealthManagerError::RecordSoftDeleted
    )]
    pub record: Account<'info, HealthRecord>,

    /// CHECK: This is the owner of the record, validated in constraints
    pub record_owner: UncheckedAccount<'info>,

    pub accessor: Signer<'info>,

    /// Optional access grant account for authorized users
    #[account(
        seeds = [ACCESS_GRANT_SEED, record_owner.key().as_ref(), accessor.key().as_ref()],
        bump = access_grant.bump,
    )]
    pub access_grant: Option<Account<'info, AccessGrant>>,
}

pub fn read_record(
    ctx: Context<ReadRecord>,
    _record_id: u64,
) -> Result<()> {
    let record = &ctx.accounts.record;
    let accessor = &ctx.accounts.accessor;
    let record_owner = &ctx.accounts.record_owner;

    // Check if accessor is owner or has valid read access
    if accessor.key() != record_owner.key() {
        let access_grant = ctx.accounts.access_grant.as_ref()
            .ok_or(HealthManagerError::UnauthorizedAccess)?;

        require!(
            access_grant.has_permission(PERMISSION_READ),
            HealthManagerError::InsufficientPermissions
        );
    }

    // Emit event for audit trail
    emit!(AuthorizedRecordAccess {
        owner: record_owner.key(),
        record_id: record.id,
        accessor: accessor.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Health record {} accessed by: {}", record.id, accessor.key());

    // Note: The actual record data is returned via the account data
    // The client can read the record fields directly from the account
    Ok(())
}