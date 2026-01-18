use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::{EmergencyAccessConfigured, EmergencyAccessActivated, EmergencyAccessDeactivated},
    state::{UserHealthProfile, EmergencyAccess},
};

#[derive(Accounts)]
#[instruction(emergency_contact: Pubkey)]
pub struct ConfigureEmergencyAccess<'info> {
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
        space = EmergencyAccess::LEN,
        seeds = [EMERGENCY_ACCESS_SEED, owner.key().as_ref(), emergency_contact.as_ref()],
        bump
    )]
    pub emergency_access: Account<'info, EmergencyAccess>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn configure_emergency_access(
    ctx: Context<ConfigureEmergencyAccess>,
    emergency_contact: Pubkey,
) -> Result<()> {
    let emergency_access = &mut ctx.accounts.emergency_access;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Validate that owner is not setting themselves as emergency contact
    require!(
        emergency_contact != owner.key(),
        HealthManagerError::CannotGrantAccessToSelf
    );

    // Initialize emergency access
    **emergency_access = EmergencyAccess::new(
        owner.key(),
        emergency_contact,
        ctx.bumps.emergency_access,
    )?;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(EmergencyAccessConfigured {
        owner: owner.key(),
        emergency_contact,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Emergency access configured for contact: {}", emergency_contact);
    Ok(())
}

#[derive(Accounts)]
#[instruction(emergency_contact: Pubkey, reason: String)]
pub struct ActivateEmergencyAccess<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [EMERGENCY_ACCESS_SEED, owner.key().as_ref(), emergency_contact.as_ref()],
        bump = emergency_access.bump,
        constraint = emergency_access.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub emergency_access: Account<'info, EmergencyAccess>,

    /// CHECK: This is the owner of the records, validated in constraints
    pub owner: UncheckedAccount<'info>,

    pub activator: Signer<'info>,
}

pub fn activate_emergency_access(
    ctx: Context<ActivateEmergencyAccess>,
    emergency_contact: Pubkey,
    reason: String,
) -> Result<()> {
    let emergency_access = &mut ctx.accounts.emergency_access;
    let profile = &mut ctx.accounts.profile;
    let activator = &ctx.accounts.activator;
    let owner = &ctx.accounts.owner;

    // Check if emergency access is already active
    require!(
        !emergency_access.is_active,
        HealthManagerError::EmergencyAccessAlreadyActive
    );

    // Only the emergency contact or the owner can activate
    require!(
        activator.key() == emergency_contact || activator.key() == owner.key(),
        HealthManagerError::UnauthorizedAccess
    );

    // Activate emergency access
    emergency_access.activate(reason.clone(), activator.key())?;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(EmergencyAccessActivated {
        owner: owner.key(),
        emergency_contact,
        activator: activator.key(),
        reason,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Emergency access activated by: {}", activator.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(emergency_contact: Pubkey)]
pub struct DeactivateEmergencyAccess<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [EMERGENCY_ACCESS_SEED, owner.key().as_ref(), emergency_contact.as_ref()],
        bump = emergency_access.bump,
        constraint = emergency_access.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub emergency_access: Account<'info, EmergencyAccess>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn deactivate_emergency_access(
    ctx: Context<DeactivateEmergencyAccess>,
    emergency_contact: Pubkey,
) -> Result<()> {
    let emergency_access = &mut ctx.accounts.emergency_access;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Deactivate emergency access
    emergency_access.deactivate();

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(EmergencyAccessDeactivated {
        owner: owner.key(),
        emergency_contact,
        deactivator: owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Emergency access deactivated for contact: {}", emergency_contact);
    Ok(())
}

#[derive(Accounts)]
#[instruction(record_id: u64, emergency_contact: Pubkey)]
pub struct AccessWithEmergency<'info> {
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
    pub record: Account<'info, crate::state::HealthRecord>,

    #[account(
        mut,
        seeds = [EMERGENCY_ACCESS_SEED, record_owner.key().as_ref(), emergency_contact.as_ref()],
        bump = emergency_access.bump,
        constraint = emergency_access.owner == record_owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = emergency_access.emergency_contact == accessor.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub emergency_access: Account<'info, EmergencyAccess>,

    /// CHECK: This is the owner of the record, validated in constraints
    pub record_owner: UncheckedAccount<'info>,

    pub accessor: Signer<'info>,
}

pub fn access_with_emergency(
    ctx: Context<AccessWithEmergency>,
    record_id: u64,
    _emergency_contact: Pubkey,
) -> Result<()> {
    let emergency_access = &mut ctx.accounts.emergency_access;
    let _record = &ctx.accounts.record;
    let accessor = &ctx.accounts.accessor;
    let record_owner = &ctx.accounts.record_owner;

    // Check if emergency access expired and clean up if needed
    emergency_access.check_and_expire();

    // Validate emergency access is active and valid
    require!(
        emergency_access.is_valid(),
        HealthManagerError::AccessGrantExpired
    );

    // Emit event for audit trail
    emit!(crate::events::AuthorizedRecordAccess {
        owner: record_owner.key(),
        record_id,
        accessor: accessor.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Record {} accessed via emergency access by: {}", record_id, accessor.key());
    Ok(())
}