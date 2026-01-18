use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::{SyncStateInitialized, SyncOperationCompleted},
    state::{UserHealthProfile, SyncState, SyncOperation},
};

#[derive(Accounts)]
#[instruction(device_id: String, encrypted_sync_key: [u8; 32], is_primary: bool)]
pub struct InitializeSyncState<'info> {
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
        space = SyncState::LEN,
        seeds = [SYNC_STATE_SEED, owner.key().as_ref(), device_id.as_bytes()],
        bump
    )]
    pub sync_state: Account<'info, SyncState>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_sync_state(
    ctx: Context<InitializeSyncState>,
    device_id: String,
    encrypted_sync_key: [u8; MAX_SYNC_KEY_LENGTH],
    is_primary: bool,
) -> Result<()> {
    let sync_state = &mut ctx.accounts.sync_state;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Initialize sync state
    **sync_state = SyncState::new(
        owner.key(),
        device_id.clone(),
        encrypted_sync_key,
        is_primary,
        ctx.bumps.sync_state,
    )?;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(SyncStateInitialized {
        owner: owner.key(),
        device_id,
        sync_version: sync_state.sync_version,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Sync state initialized for device: {}", sync_state.device_id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(source_device: String, target_device: String, operation_type: String, sync_key: [u8; 32])]
pub struct StartSyncOperation<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [SYNC_STATE_SEED, owner.key().as_ref(), source_device.as_bytes()],
        bump = source_sync_state.bump,
        constraint = source_sync_state.owner == owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = source_sync_state.validate_sync_key(&sync_key) @ HealthManagerError::SyncKeyMismatch
    )]
    pub source_sync_state: Account<'info, SyncState>,

    #[account(
        mut,
        seeds = [SYNC_STATE_SEED, owner.key().as_ref(), target_device.as_bytes()],
        bump = target_sync_state.bump,
        constraint = target_sync_state.owner == owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = target_sync_state.validate_sync_key(&sync_key) @ HealthManagerError::SyncKeyMismatch
    )]
    pub target_sync_state: Account<'info, SyncState>,

    #[account(
        init,
        payer = owner,
        space = SyncOperation::LEN,
        seeds = [SYNC_STATE_SEED, b"operation", owner.key().as_ref(), source_device.as_bytes(), target_device.as_bytes()],
        bump
    )]
    pub sync_operation: Account<'info, SyncOperation>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn start_sync_operation(
    ctx: Context<StartSyncOperation>,
    source_device: String,
    target_device: String,
    operation_type: String,
    _sync_key: [u8; MAX_SYNC_KEY_LENGTH],
) -> Result<()> {
    let sync_operation = &mut ctx.accounts.sync_operation;
    let source_sync_state = &ctx.accounts.source_sync_state;
    let target_sync_state = &ctx.accounts.target_sync_state;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Check for sync conflicts
    let has_conflict = source_sync_state.detect_conflict(target_sync_state.state_hash);
    if has_conflict && operation_type != "conflict_resolution" {
        return Err(HealthManagerError::SyncConflict.into());
    }

    // Initialize sync operation
    **sync_operation = SyncOperation::new(
        owner.key(),
        source_device,
        target_device,
        operation_type,
        source_sync_state.sync_version,
        target_sync_state.sync_version,
        ctx.bumps.sync_operation,
    )?;

    // Update profile timestamp
    profile.update_timestamp();

    msg!("Sync operation started between devices");
    Ok(())
}

#[derive(Accounts)]
#[instruction(source_device: String, target_device: String, records_synced: u64, new_state_hash: [u8; 32])]
pub struct CompleteSyncOperation<'info> {
    #[account(
        mut,
        seeds = [SYNC_STATE_SEED, b"operation", owner.key().as_ref(), source_device.as_bytes(), target_device.as_bytes()],
        bump = sync_operation.bump,
        constraint = sync_operation.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub sync_operation: Account<'info, SyncOperation>,

    #[account(
        mut,
        seeds = [SYNC_STATE_SEED, owner.key().as_ref(), sync_operation.source_device.as_bytes()],
        bump = source_sync_state.bump,
        constraint = source_sync_state.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub source_sync_state: Account<'info, SyncState>,

    #[account(
        mut,
        seeds = [SYNC_STATE_SEED, owner.key().as_ref(), sync_operation.target_device.as_bytes()],
        bump = target_sync_state.bump,
        constraint = target_sync_state.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub target_sync_state: Account<'info, SyncState>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn complete_sync_operation(
    ctx: Context<CompleteSyncOperation>,
    _source_device: String,
    _target_device: String,
    records_synced: u64,
    new_state_hash: [u8; 32],
) -> Result<()> {
    let sync_operation = &mut ctx.accounts.sync_operation;
    let source_sync_state = &mut ctx.accounts.source_sync_state;
    let target_sync_state = &mut ctx.accounts.target_sync_state;
    let owner = &ctx.accounts.owner;

    // Complete the sync operation
    sync_operation.complete_successfully(records_synced)?;

    // Update sync states
    source_sync_state.update_sync_state(new_state_hash)?;
    target_sync_state.update_sync_state(new_state_hash)?;

    // Emit event
    emit!(SyncOperationCompleted {
        owner: owner.key(),
        source_device: sync_operation.source_device.clone(),
        target_device: sync_operation.target_device.clone(),
        sync_version: source_sync_state.sync_version,
        records_synced,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Sync operation completed successfully");
    Ok(())
}

#[derive(Accounts)]
#[instruction(source_device: String, target_device: String, error_message: String)]
pub struct FailSyncOperation<'info> {
    #[account(
        mut,
        seeds = [SYNC_STATE_SEED, b"operation", owner.key().as_ref(), source_device.as_bytes(), target_device.as_bytes()],
        bump = sync_operation.bump,
        constraint = sync_operation.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub sync_operation: Account<'info, SyncOperation>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn fail_sync_operation(
    ctx: Context<FailSyncOperation>,
    _source_device: String,
    _target_device: String,
    error_message: String,
) -> Result<()> {
    let sync_operation = &mut ctx.accounts.sync_operation;

    // Mark operation as failed
    sync_operation.complete_with_error(error_message)?;

    msg!("Sync operation failed");
    Ok(())
}

#[derive(Accounts)]
#[instruction(device_id: String, is_primary: bool)]
pub struct UpdateSyncPrimary<'info> {
    #[account(
        mut,
        seeds = [SYNC_STATE_SEED, owner.key().as_ref(), device_id.as_bytes()],
        bump = sync_state.bump,
        constraint = sync_state.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub sync_state: Account<'info, SyncState>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn update_sync_primary(
    ctx: Context<UpdateSyncPrimary>,
    _device_id: String,
    is_primary: bool,
) -> Result<()> {
    let sync_state = &mut ctx.accounts.sync_state;

    // Update primary status
    sync_state.set_primary(is_primary);

    msg!("Sync primary status updated");
    Ok(())
}