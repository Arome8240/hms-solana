use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct WearableDevice {
    /// Owner of the wearable device
    pub owner: Pubkey,
    /// Unique device identifier
    pub device_id: String,
    /// Type of wearable device (e.g., "smartwatch", "fitness_tracker")
    pub device_type: String,
    /// Device's public key for signature verification
    pub device_pubkey: Pubkey,
    /// Whether the device is currently active
    pub is_active: bool,
    /// Last time device sent data
    pub last_data_timestamp: i64,
    /// Total number of data points received
    pub data_points_received: u64,
    /// Timestamp when device was registered
    pub registered_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl WearableDevice {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + MAX_DEVICE_ID_LENGTH + // device_id
        4 + MAX_RECORD_TYPE_LENGTH + // device_type
        32 + // device_pubkey
        1 +  // is_active
        8 +  // last_data_timestamp
        8 +  // data_points_received
        8 +  // registered_at
        1;   // bump

    pub fn new(
        owner: Pubkey,
        device_id: String,
        device_type: String,
        device_pubkey: Pubkey,
        bump: u8,
    ) -> Result<Self> {
        require!(
            device_id.len() <= MAX_DEVICE_ID_LENGTH,
            crate::errors::HealthManagerError::DeviceIdTooLong
        );
        require!(
            device_type.len() <= MAX_RECORD_TYPE_LENGTH,
            crate::errors::HealthManagerError::RecordTypeTooLong
        );

        let now = Clock::get()?.unix_timestamp;

        Ok(Self {
            owner,
            device_id,
            device_type,
            device_pubkey,
            is_active: true,
            last_data_timestamp: 0,
            data_points_received: 0,
            registered_at: now,
            bump,
        })
    }

    pub fn record_data_ingestion(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        self.last_data_timestamp = now;
        self.data_points_received += 1;
        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn reactivate(&mut self) {
        self.is_active = true;
    }

    pub fn is_data_fresh(&self, max_age_seconds: i64) -> bool {
        if self.last_data_timestamp == 0 {
            return false;
        }

        let now = Clock::get().unwrap().unix_timestamp;
        now - self.last_data_timestamp <= max_age_seconds
    }
}

#[account]
pub struct WearableDataBatch {
    /// Owner of the data
    pub owner: Pubkey,
    /// Device that generated the data
    pub device_id: String,
    /// Batch identifier
    pub batch_id: u64,
    /// Encrypted data URI (IPFS/Arweave)
    pub encrypted_data_uri: String,
    /// Hash of the raw data for integrity
    pub data_hash: [u8; 32],
    /// Number of data points in this batch
    pub data_point_count: u32,
    /// Timestamp of the first data point
    pub start_timestamp: i64,
    /// Timestamp of the last data point
    pub end_timestamp: i64,
    /// When this batch was created on-chain
    pub created_at: i64,
    /// Whether this batch has been processed
    pub is_processed: bool,
    /// PDA bump
    pub bump: u8,
}

impl WearableDataBatch {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + MAX_DEVICE_ID_LENGTH + // device_id
        8 +  // batch_id
        4 + MAX_ENCRYPTED_URI_LENGTH + // encrypted_data_uri
        32 + // data_hash
        4 +  // data_point_count
        8 +  // start_timestamp
        8 +  // end_timestamp
        8 +  // created_at
        1 +  // is_processed
        1;   // bump

    pub fn new(
        owner: Pubkey,
        device_id: String,
        batch_id: u64,
        encrypted_data_uri: String,
        data_hash: [u8; 32],
        data_point_count: u32,
        start_timestamp: i64,
        end_timestamp: i64,
        bump: u8,
    ) -> Result<Self> {
        require!(
            device_id.len() <= MAX_DEVICE_ID_LENGTH,
            crate::errors::HealthManagerError::DeviceIdTooLong
        );
        require!(
            encrypted_data_uri.len() <= MAX_ENCRYPTED_URI_LENGTH,
            crate::errors::HealthManagerError::EncryptedUriTooLong
        );

        let now = Clock::get()?.unix_timestamp;

        // Validate data isn't too old
        require!(
            now - end_timestamp <= WEARABLE_DATA_RETENTION,
            crate::errors::HealthManagerError::WearableDataTooOld
        );

        Ok(Self {
            owner,
            device_id,
            batch_id,
            encrypted_data_uri,
            data_hash,
            data_point_count,
            start_timestamp,
            end_timestamp,
            created_at: now,
            is_processed: false,
            bump,
        })
    }

    pub fn mark_processed(&mut self) {
        self.is_processed = true;
    }
}