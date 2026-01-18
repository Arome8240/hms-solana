use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct HealthRecord {
    /// Unique identifier for this record within the user's profile
    pub id: u64,
    /// Owner of this health record
    pub owner: Pubkey,
    /// Type of health record (e.g., "vital", "appointment", "lab", "medication")
    pub record_type: String,
    /// IPFS/Arweave CID or encrypted reference to off-chain data
    pub encrypted_uri: String,
    /// SHA-256 hash of the original data for integrity verification
    pub data_hash: [u8; 32],
    /// Timestamp when record was created
    pub timestamp: i64,
    /// Timestamp when record was last updated
    pub last_updated: i64,
    /// Optional metadata (encrypted or public depending on use case)
    pub metadata: String,
    /// Soft delete flag - record remains on-chain but marked as deleted
    pub is_deleted: bool,
    /// PDA bump for validation
    pub bump: u8,
}

impl HealthRecord {
    pub const LEN: usize = 8 + // discriminator
        8 +  // id
        32 + // owner
        4 + MAX_RECORD_TYPE_LENGTH + // record_type (String)
        4 + MAX_ENCRYPTED_URI_LENGTH + // encrypted_uri (String)
        32 + // data_hash
        8 +  // timestamp
        8 +  // last_updated
        4 + MAX_METADATA_LENGTH + // metadata (String)
        1 +  // is_deleted
        1;   // bump

    pub fn new(
        id: u64,
        owner: Pubkey,
        record_type: String,
        encrypted_uri: String,
        data_hash: [u8; 32],
        metadata: String,
        bump: u8,
    ) -> Result<Self> {
        require!(
            record_type.len() <= MAX_RECORD_TYPE_LENGTH,
            crate::errors::HealthManagerError::RecordTypeTooLong
        );
        require!(
            encrypted_uri.len() <= MAX_ENCRYPTED_URI_LENGTH,
            crate::errors::HealthManagerError::EncryptedUriTooLong
        );
        require!(
            metadata.len() <= MAX_METADATA_LENGTH,
            crate::errors::HealthManagerError::MetadataTooLong
        );

        let now = Clock::get()?.unix_timestamp;
        Ok(Self {
            id,
            owner,
            record_type,
            encrypted_uri,
            data_hash,
            timestamp: now,
            last_updated: now,
            metadata,
            is_deleted: false,
            bump,
        })
    }

    pub fn update_metadata(&mut self, metadata: String) -> Result<()> {
        require!(
            metadata.len() <= MAX_METADATA_LENGTH,
            crate::errors::HealthManagerError::MetadataTooLong
        );

        self.metadata = metadata;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn soft_delete(&mut self) {
        self.is_deleted = true;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }

    pub fn is_accessible(&self) -> bool {
        !self.is_deleted
    }
}