use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct SyncState {
    /// Owner of the sync state
    pub owner: Pubkey,
    /// Device identifier for this sync state
    pub device_id: String,
    /// Current sync version/sequence number
    pub sync_version: u64,
    /// Encrypted sync key for cross-device encryption
    pub encrypted_sync_key: [u8; MAX_SYNC_KEY_LENGTH],
    /// Hash of the current state for conflict detection
    pub state_hash: [u8; 32],
    /// Last successful sync timestamp
    pub last_sync_timestamp: i64,
    /// Number of successful syncs
    pub sync_count: u64,
    /// Whether this device is the primary sync source
    pub is_primary: bool,
    /// Timestamp when sync state was created
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl SyncState {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + MAX_DEVICE_ID_LENGTH + // device_id
        8 +  // sync_version
        MAX_SYNC_KEY_LENGTH + // encrypted_sync_key
        32 + // state_hash
        8 +  // last_sync_timestamp
        8 +  // sync_count
        1 +  // is_primary
        8 +  // created_at
        1;   // bump

    pub fn new(
        owner: Pubkey,
        device_id: String,
        encrypted_sync_key: [u8; MAX_SYNC_KEY_LENGTH],
        is_primary: bool,
        bump: u8,
    ) -> Result<Self> {
        require!(
            device_id.len() <= MAX_DEVICE_ID_LENGTH,
            crate::errors::HealthManagerError::DeviceIdTooLong
        );

        let now = Clock::get()?.unix_timestamp;

        Ok(Self {
            owner,
            device_id,
            sync_version: 1,
            encrypted_sync_key,
            state_hash: [0u8; 32], // Will be updated on first sync
            last_sync_timestamp: 0,
            sync_count: 0,
            is_primary,
            created_at: now,
            bump,
        })
    }

    pub fn update_sync_state(
        &mut self,
        new_state_hash: [u8; 32],
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        self.sync_version += 1;
        self.state_hash = new_state_hash;
        self.last_sync_timestamp = now;
        self.sync_count += 1;

        Ok(())
    }

    pub fn validate_sync_key(&self, provided_key: &[u8; MAX_SYNC_KEY_LENGTH]) -> bool {
        self.encrypted_sync_key == *provided_key
    }

    pub fn detect_conflict(&self, other_state_hash: [u8; 32]) -> bool {
        self.state_hash != other_state_hash
    }

    pub fn set_primary(&mut self, is_primary: bool) {
        self.is_primary = is_primary;
    }
}

#[account]
pub struct SyncOperation {
    /// Owner of the sync operation
    pub owner: Pubkey,
    /// Source device for the sync
    pub source_device: String,
    /// Target device for the sync
    pub target_device: String,
    /// Operation type (e.g., "full_sync", "incremental", "conflict_resolution")
    pub operation_type: String,
    /// Source sync version
    pub source_version: u64,
    /// Target sync version
    pub target_version: u64,
    /// Number of records synced
    pub records_synced: u64,
    /// Whether the operation completed successfully
    pub is_successful: bool,
    /// Error message if operation failed
    pub error_message: String,
    /// Timestamp when operation started
    pub started_at: i64,
    /// Timestamp when operation completed
    pub completed_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl SyncOperation {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + MAX_DEVICE_ID_LENGTH + // source_device
        4 + MAX_DEVICE_ID_LENGTH + // target_device
        4 + MAX_RECORD_TYPE_LENGTH + // operation_type
        8 +  // source_version
        8 +  // target_version
        8 +  // records_synced
        1 +  // is_successful
        4 + MAX_METADATA_LENGTH + // error_message
        8 +  // started_at
        8 +  // completed_at
        1;   // bump

    pub fn new(
        owner: Pubkey,
        source_device: String,
        target_device: String,
        operation_type: String,
        source_version: u64,
        target_version: u64,
        bump: u8,
    ) -> Result<Self> {
        require!(
            source_device.len() <= MAX_DEVICE_ID_LENGTH,
            crate::errors::HealthManagerError::DeviceIdTooLong
        );
        require!(
            target_device.len() <= MAX_DEVICE_ID_LENGTH,
            crate::errors::HealthManagerError::DeviceIdTooLong
        );
        require!(
            operation_type.len() <= MAX_RECORD_TYPE_LENGTH,
            crate::errors::HealthManagerError::RecordTypeTooLong
        );

        let now = Clock::get()?.unix_timestamp;

        Ok(Self {
            owner,
            source_device,
            target_device,
            operation_type,
            source_version,
            target_version,
            records_synced: 0,
            is_successful: false,
            error_message: String::new(),
            started_at: now,
            completed_at: 0,
            bump,
        })
    }

    pub fn complete_successfully(&mut self, records_synced: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        self.records_synced = records_synced;
        self.is_successful = true;
        self.completed_at = now;

        Ok(())
    }

    pub fn complete_with_error(&mut self, error_message: String) -> Result<()> {
        require!(
            error_message.len() <= MAX_METADATA_LENGTH,
            crate::errors::HealthManagerError::MetadataTooLong
        );

        let now = Clock::get()?.unix_timestamp;

        self.is_successful = false;
        self.error_message = error_message;
        self.completed_at = now;

        Ok(())
    }
}