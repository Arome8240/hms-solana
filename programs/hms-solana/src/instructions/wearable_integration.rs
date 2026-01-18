use anchor_lang::prelude::*;
use crate::{
    constants::*,
    errors::HealthManagerError,
    events::{WearableDeviceRegistered, WearableDataIngested},
    state::{UserHealthProfile, WearableDevice, WearableDataBatch, HealthRecord},
};

#[derive(Accounts)]
#[instruction(device_id: String, device_type: String, device_pubkey: Pubkey)]
pub struct RegisterWearableDevice<'info> {
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
        space = WearableDevice::LEN,
        seeds = [WEARABLE_DEVICE_SEED, owner.key().as_ref(), device_id.as_bytes()],
        bump
    )]
    pub wearable_device: Account<'info, WearableDevice>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn register_wearable_device(
    ctx: Context<RegisterWearableDevice>,
    device_id: String,
    device_type: String,
    device_pubkey: Pubkey,
) -> Result<()> {
    let wearable_device = &mut ctx.accounts.wearable_device;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Initialize the wearable device
    **wearable_device = WearableDevice::new(
        owner.key(),
        device_id.clone(),
        device_type.clone(),
        device_pubkey,
        ctx.bumps.wearable_device,
    )?;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(WearableDeviceRegistered {
        owner: owner.key(),
        device_id,
        device_type,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Wearable device registered for user: {}", owner.key());
    Ok(())
}

#[derive(Accounts)]
#[instruction(device_id: String, batch_id: u64, encrypted_data_uri: String, data_hash: [u8; 32], data_point_count: u32, start_timestamp: i64, end_timestamp: i64)]
pub struct IngestWearableData<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [WEARABLE_DEVICE_SEED, owner.key().as_ref(), device_id.as_bytes()],
        bump = wearable_device.bump,
        constraint = wearable_device.owner == owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = wearable_device.is_active @ HealthManagerError::WearableDeviceNotRegistered
    )]
    pub wearable_device: Account<'info, WearableDevice>,

    #[account(
        init,
        payer = owner,
        space = WearableDataBatch::LEN,
        seeds = [WEARABLE_DEVICE_SEED, b"batch", owner.key().as_ref(), device_id.as_bytes(), batch_id.to_le_bytes().as_ref()],
        bump
    )]
    pub data_batch: Account<'info, WearableDataBatch>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn ingest_wearable_data(
    ctx: Context<IngestWearableData>,
    device_id: String,
    batch_id: u64,
    encrypted_data_uri: String,
    data_hash: [u8; 32],
    data_point_count: u32,
    start_timestamp: i64,
    end_timestamp: i64,
) -> Result<()> {
    let data_batch = &mut ctx.accounts.data_batch;
    let wearable_device = &mut ctx.accounts.wearable_device;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Initialize the data batch
    **data_batch = WearableDataBatch::new(
        owner.key(),
        device_id.clone(),
        batch_id,
        encrypted_data_uri,
        data_hash,
        data_point_count,
        start_timestamp,
        end_timestamp,
        ctx.bumps.data_batch,
    )?;

    // Update wearable device with data ingestion
    wearable_device.record_data_ingestion()?;

    // Update profile timestamp
    profile.update_timestamp();

    // Emit event
    emit!(WearableDataIngested {
        owner: owner.key(),
        device_id: device_id.clone(),
        data_type: "batch".to_string(),
        record_id: batch_id,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Wearable data batch {} ingested for device: {}", batch_id, device_id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(device_id: String, batch_id: u64, record_type: String, metadata: String)]
pub struct ProcessWearableDataToRecord<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [WEARABLE_DEVICE_SEED, b"batch", owner.key().as_ref(), device_id.as_bytes(), batch_id.to_le_bytes().as_ref()],
        bump = data_batch.bump,
        constraint = data_batch.owner == owner.key() @ HealthManagerError::UnauthorizedAccess,
        constraint = !data_batch.is_processed @ HealthManagerError::InvalidSyncOperation
    )]
    pub data_batch: Account<'info, WearableDataBatch>,

    #[account(
        init,
        payer = owner,
        space = HealthRecord::LEN,
        seeds = [HEALTH_RECORD_SEED, owner.key().as_ref(), profile.record_count.to_le_bytes().as_ref()],
        bump
    )]
    pub health_record: Account<'info, HealthRecord>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process_wearable_data_to_record(
    ctx: Context<ProcessWearableDataToRecord>,
    _device_id: String,
    _batch_id: u64,
    record_type: String,
    metadata: String,
) -> Result<()> {
    let data_batch = &mut ctx.accounts.data_batch;
    let health_record = &mut ctx.accounts.health_record;
    let profile = &mut ctx.accounts.profile;
    let owner = &ctx.accounts.owner;

    // Create health record from wearable data
    **health_record = HealthRecord::new(
        profile.record_count,
        owner.key(),
        record_type.clone(),
        data_batch.encrypted_data_uri.clone(),
        data_batch.data_hash,
        metadata,
        ctx.bumps.health_record,
    )?;

    // Mark data batch as processed
    data_batch.mark_processed();

    // Update profile
    profile.increment_record_count();

    // Emit event
    emit!(crate::events::HealthRecordAdded {
        owner: owner.key(),
        record_id: health_record.id,
        record_type,
        actor: owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Wearable data processed into health record: {}", health_record.id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(device_id: String)]
pub struct DeactivateWearableDevice<'info> {
    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, owner.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub profile: Account<'info, UserHealthProfile>,

    #[account(
        mut,
        seeds = [WEARABLE_DEVICE_SEED, owner.key().as_ref(), device_id.as_bytes()],
        bump = wearable_device.bump,
        constraint = wearable_device.owner == owner.key() @ HealthManagerError::UnauthorizedAccess
    )]
    pub wearable_device: Account<'info, WearableDevice>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn deactivate_wearable_device(
    ctx: Context<DeactivateWearableDevice>,
    _device_id: String,
) -> Result<()> {
    let wearable_device = &mut ctx.accounts.wearable_device;
    let profile = &mut ctx.accounts.profile;

    // Deactivate the device
    wearable_device.deactivate();

    // Update profile timestamp
    profile.update_timestamp();

    msg!("Wearable device deactivated");
    Ok(())
}